use std::{
    collections::VecDeque,
    sync::mpsc::{Receiver, RecvError, Sender},
};

use super::{IgnoreOptions, ProcessManager, ProcessSearchResults};

pub enum Operations {
    Search(String),
    KillProcess(u32),
    Shutdown,
}

pub enum OperationResult {
    ProcessKilled(ProcessSearchResults),
    ProcessKillFailed,
    SearchCompleted(ProcessSearchResults),
    Error(String),
}

pub fn start(
    process_manager: ProcessManager,
    ignore_options: IgnoreOptions,
) -> (Sender<Operations>, Receiver<OperationResult>) {
    let (operations_sender, operations_reveiver) = std::sync::mpsc::channel();
    let (result_sender, result_reveiver) = std::sync::mpsc::channel();
    std::thread::spawn(|| {
        process_loop(
            process_manager,
            operations_reveiver,
            ignore_options,
            result_sender,
        );
    });
    (operations_sender, result_reveiver)
}

fn process_loop(
    mut process_manager: ProcessManager,
    operations_reveiver: Receiver<Operations>,
    ignore_options: IgnoreOptions,
    result_sender: Sender<OperationResult>,
) {
    let mut last_query = String::new();
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
                    let result =
                        process_manager.refresh_and_find_processes(&query, &ignore_options);
                    send_result(OperationResult::SearchCompleted(result), &result_sender);
                    last_query = query;
                }
                Operations::KillProcess(pid) => {
                    if process_manager.kill_process(pid) {
                        let mut search_results = process_manager
                            .refresh_and_find_processes(&last_query, &ignore_options);
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
