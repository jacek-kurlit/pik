use pik::processes::{FilterOptions, ProcessManager};

#[test]
fn should_find_cargo_process_by_cmd_name() {
    let mut process_manager = ProcessManager::new().unwrap();
    let results = process_manager.find_processes("cargo", FilterOptions::default());
    assert!(!results.is_empty());
    assert!(results.iter().all(|p| p.cmd.contains("cargo")));
}

#[test]
fn should_find_cargo_process_by_cmd_path() {
    let mut process_manager = ProcessManager::new().unwrap();
    let results = process_manager.find_processes("/cargo", FilterOptions::default());
    assert!(!results.is_empty());
    assert!(results
        .iter()
        .all(|p| p.cmd_path.as_ref().unwrap().contains("cargo")));
}

#[test]
fn should_find_cargo_process_by_name_path_or_args() {
    let mut process_manager = ProcessManager::new().unwrap();
    let results = process_manager.find_processes("~cargo", FilterOptions::default());
    assert!(!results.is_empty());
    assert!(results
        .iter()
        .all(|p| p.cmd_path.as_ref().unwrap().contains("cargo")
            || p.args.contains("cargo")
            || p.cmd.contains("cargo")));
}

#[test]
fn should_find_cargo_process_by_args() {
    let mut process_manager = ProcessManager::new().unwrap();
    let results = process_manager.find_processes("-test", FilterOptions::default());
    assert!(!results.is_empty());
    assert!(results.iter().all(|p| p.args.contains("test")));
}

use http_test_server::TestServer;
#[test]
fn should_find_cargo_process_by_port() {
    let test_server = TestServer::new().unwrap();
    let port = test_server.port();
    let mut process_manager = ProcessManager::new().unwrap();
    let results = process_manager.find_processes(&format!(":{}", port), FilterOptions::default());
    assert!(!results.is_empty());
    assert!(results.iter().all(|p| p.ports == Some(format!("{}", port))));
}