use std::sync::mpsc::{Receiver, Sender};

use super::{IgnoreOptions, ProcessManager, ProcessSearchResults};

pub enum Operations {
    Search(String),
    KillProcess(u32, String),
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
    loop {
        let operation = operations_reveiver.recv();
        if let Err(err) = operation {
            send_result(
                OperationResult::Error(format!("Daemon received error from channel : {err}")),
                &result_sender,
            );
            break;
        }
        match operation.unwrap() {
            Operations::Search(query) => {
                let result = process_manager.find_processes(&query, &ignore_options);
                send_result(OperationResult::SearchCompleted(result), &result_sender);
            }
            Operations::KillProcess(pid, query) => {
                if process_manager.kill_process(pid) {
                    let mut search_results =
                        process_manager.find_processes(&query, &ignore_options);
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
            Operations::Shutdown => break,
        }
    }
}

fn send_result(result: OperationResult, result_sender: &Sender<OperationResult>) {
    result_sender
        .send(result)
        .expect("Failed to send result, cannot continue (connetion was closed?)");
}
