use std::{
    collections::VecDeque,
    sync::mpsc::{Receiver, RecvError, Sender},
};

use anyhow::Result;

use super::{IgnoreOptions, ProcessManager, ProcessSearchResults};

pub struct ProcssAsyncService {
    process_manager: ProcessManager,
    ignore_options: IgnoreOptions,
    last_query: String,
}

impl ProcssAsyncService {
    pub fn new(process_manager: ProcessManager, ignore_options: IgnoreOptions) -> Self {
        Self {
            process_manager,
            ignore_options,
            last_query: String::new(),
        }
    }

    pub fn find_processes(&mut self, query: &str) -> ProcessSearchResults {
        self.last_query = query.to_string();
        self.process_manager
            .find_processes(query, &self.ignore_options)
    }

    pub fn run_as_background_process(self) -> (Sender<Operations>, Receiver<OperationResult>) {
        let (operations_sender, operations_reveiver) = std::sync::mpsc::channel();
        let (result_sender, result_reveiver) = std::sync::mpsc::channel();
        std::thread::spawn(|| {
            process_loop(self, operations_reveiver, result_sender);
        });
        (operations_sender, result_reveiver)
    }

    fn refresh_and_find_processes(&mut self, query: &str) -> ProcessSearchResults {
        self.process_manager.refresh();
        self.find_processes(query)
    }

    fn rerun_last_search(&mut self) -> ProcessSearchResults {
        self.process_manager.refresh();
        self.process_manager
            .find_processes(&self.last_query, &self.ignore_options)
    }
}

pub enum Operations {
    Search(String),
    KillProcess(u32),
    Shutdown,
}

#[derive(Debug)]
pub enum OperationResult {
    ProcessKilled(ProcessSearchResults),
    ProcessKillFailed,
    SearchCompleted(ProcessSearchResults),
    Error(String),
}

fn process_loop(
    mut service: ProcssAsyncService,
    operations_reveiver: Receiver<Operations>,
    result_sender: Sender<OperationResult>,
) {
    loop {
        let operations = receive_operations(&operations_reveiver);
        if let Err(err) = operations {
            send_result(
                OperationResult::Error(format!("Daemon received error from channel : {err}")),
                &result_sender,
            );
            break;
        }
        for operation in operations.unwrap() {
            match operation {
                Operations::Search(query) => {
                    let result = service.refresh_and_find_processes(&query);
                    send_result(OperationResult::SearchCompleted(result), &result_sender);
                }
                Operations::KillProcess(pid) => {
                    if service.process_manager.kill_process(pid) {
                        let mut search_results = service.rerun_last_search();
                        //NOTE: cache refresh takes time and process may reappear in list!
                        search_results.remove(pid);
                        send_result(
                            OperationResult::ProcessKilled(search_results),
                            &result_sender,
                        );
                    } else {
                        send_result(OperationResult::ProcessKillFailed, &result_sender);
                    }
                }
                Operations::Shutdown => {
                    return;
                }
            }
        }
    }
}

// Receive operations from the channel, coalesce multiple search operations into one
fn receive_operations(
    operations_reveiver: &Receiver<Operations>,
) -> Result<VecDeque<Operations>, RecvError> {
    //

    let mut stack = VecDeque::new();
    stack.push_back(operations_reveiver.recv()?);

    while let Ok(next_operation) = operations_reveiver.try_recv() {
        if matches!(&stack.back(), Some(Operations::Search(_)))
            && matches!(&next_operation, Operations::Search(_))
        {
            stack.pop_back();
        }
        stack.push_back(next_operation);
    }
    Ok(stack)
}

fn send_result(result: OperationResult, result_sender: &Sender<OperationResult>) {
    result_sender
        .send(result)
        .expect("Failed to send result, cannot continue (connetion was closed?)");
}

#[cfg(test)]
mod tests {
    use std::{sync::mpsc::RecvTimeoutError, time::Duration};

    use crate::processes::{
        IgnoreOptions, ProcessManager, ProcessSearchResults, ProcssAsyncService,
    };

    #[test]
    fn find_processes_remembers_last_search() {
        // given
        let ignore_options = IgnoreOptions::default();
        let mut process_manager = ProcessManager::faux();
        faux::when!(process_manager.find_processes("query", ignore_options))
            .then(|_| ProcessSearchResults::empty());

        let mut service = ProcssAsyncService::new(process_manager, IgnoreOptions::default());

        // when
        let actual = service.find_processes("query");

        // then
        assert_eq!(service.last_query, "query");
        assert!(actual.is_empty());
    }

    #[test]
    fn should_refresh_and_find_processes() {
        // given
        let ignore_options = IgnoreOptions::default();
        let mut process_manager = ProcessManager::faux();
        faux::when!(process_manager.find_processes("query", ignore_options))
            .then(|_| ProcessSearchResults::empty());
        faux::when!(process_manager.refresh()).once().then(|_| {});

        let mut service = ProcssAsyncService::new(process_manager, IgnoreOptions::default());

        // when
        let actual = service.refresh_and_find_processes("query");

        // then
        assert_eq!(service.last_query, "query");
        assert!(actual.is_empty());
    }

    #[test]
    fn should_rereun_last_search() {
        // given
        let ignore_options = IgnoreOptions::default();
        let mut process_manager = ProcessManager::faux();
        faux::when!(process_manager.find_processes("last_query", ignore_options))
            .then(|_| ProcessSearchResults::empty());
        faux::when!(process_manager.refresh()).once().then(|_| {});

        let mut service = ProcssAsyncService::new(process_manager, IgnoreOptions::default());
        service.last_query = "last_query".to_string();

        // when
        let actual = service.rerun_last_search();

        // then
        assert_eq!(service.last_query, "last_query");
        assert!(actual.is_empty());
    }

    #[test]
    fn should_handle_background_search_operation() {
        // given
        let ignore_options = IgnoreOptions::default();
        let mut process_manager = ProcessManager::faux();
        faux::when!(process_manager.find_processes("query", ignore_options))
            .then(|_| ProcessSearchResults::empty());
        faux::when!(process_manager.refresh()).once().then(|_| {});

        let (operation_sender, result_receiver) =
            ProcssAsyncService::new(process_manager, IgnoreOptions::default())
                .run_as_background_process();

        // when
        operation_sender
            .send(crate::processes::Operations::Search("query".to_string()))
            .unwrap();

        // then
        let actual = result_receiver
            .recv_timeout(Duration::from_millis(500))
            .unwrap();
        assert!(matches!(
            actual,
            crate::processes::OperationResult::SearchCompleted(_)
        ));
    }

    #[test]
    fn should_handle_background_kill_process_operation() {
        // given
        let ignore_options = IgnoreOptions::default();
        let mut process_manager = ProcessManager::faux();
        let pid = 1000;
        faux::when!(process_manager.kill_process(pid)).then_return(true);
        faux::when!(process_manager.find_processes("", ignore_options))
            .then(|_| ProcessSearchResults::empty());
        faux::when!(process_manager.refresh()).once().then(|_| {});

        let (operation_sender, result_receiver) =
            ProcssAsyncService::new(process_manager, IgnoreOptions::default())
                .run_as_background_process();

        // when
        operation_sender
            .send(crate::processes::Operations::KillProcess(pid))
            .unwrap();

        // then
        let actual = result_receiver
            .recv_timeout(Duration::from_millis(500))
            .unwrap();
        assert!(matches!(
            actual,
            crate::processes::OperationResult::ProcessKilled(_)
        ));
    }

    #[test]
    fn should_handle_background_kill_process_fail_operation() {
        // given
        let mut process_manager = ProcessManager::faux();
        let pid = 1000;
        faux::when!(process_manager.kill_process(pid)).then_return(false);

        let (operation_sender, result_receiver) =
            ProcssAsyncService::new(process_manager, IgnoreOptions::default())
                .run_as_background_process();

        // when
        operation_sender
            .send(crate::processes::Operations::KillProcess(pid))
            .unwrap();

        // then
        let actual = result_receiver
            .recv_timeout(Duration::from_millis(500))
            .unwrap();
        assert!(matches!(
            actual,
            crate::processes::OperationResult::ProcessKillFailed
        ));
    }

    #[test]
    fn should_handle_background_kill_shutdown_operation() {
        // given
        let process_manager = ProcessManager::faux();
        let (operation_sender, result_receiver) =
            ProcssAsyncService::new(process_manager, IgnoreOptions::default())
                .run_as_background_process();

        // when
        operation_sender
            .send(crate::processes::Operations::Shutdown)
            .unwrap();

        //then
        let actual = result_receiver.recv_timeout(Duration::from_millis(500));
        assert!(actual.is_err());
        assert!(matches!(
            actual.unwrap_err(),
            RecvTimeoutError::Disconnected
        ))
    }
}
