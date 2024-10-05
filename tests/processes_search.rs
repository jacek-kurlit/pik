use std::{thread, time::Duration};

use pik::processes::{FilterOptions, ProcessManager};

use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};

#[test]
fn should_find_cargo_process_by_cmd_name() {
    let mut process_manager = ProcessManager::new().unwrap();
    let results = process_manager.find_processes("cargo", FilterOptions::default());
    assert!(!results.is_empty());
    assert!(results.iter().all(|p| fuzzy_matches(&p.cmd, "cargo")));
}

#[test]
fn should_find_cargo_process_by_cmd_path() {
    let mut process_manager = ProcessManager::new().unwrap();
    let results = process_manager.find_processes("/cargo", FilterOptions::default());
    assert!(!results.is_empty());
    assert!(results
        .iter()
        .all(|p| fuzzy_matches(p.cmd_path.as_ref().unwrap(), "cargo")));
}

#[test]
fn should_find_cargo_process_by_name_path_or_args() {
    let mut process_manager = ProcessManager::new().unwrap();
    let results = process_manager.find_processes("~cargo", FilterOptions::default());
    assert!(!results.is_empty());
    assert!(results
        .iter()
        .all(|p| fuzzy_matches(p.cmd_path.as_ref().unwrap(), "cargo")
            || p.args.contains("cargo")
            || fuzzy_matches(&p.cmd, "cargo")));
}

#[test]
fn should_find_cargo_process_by_args() {
    let mut process_manager = ProcessManager::new().unwrap();
    let results = process_manager.find_processes("-test", FilterOptions::default());
    assert!(!results.is_empty());
    assert!(results.iter().all(|p| fuzzy_matches(&p.args, "test")));
}

use http_test_server::TestServer;
#[test]
fn should_find_cargo_process_by_port() {
    let test_server = TestServer::new().unwrap();
    let port = test_server.port();
    // NOTE: Someties system needs time to notice the port is in use
    thread::sleep(Duration::from_millis(250));
    let mut process_manager = ProcessManager::new().unwrap();
    let results = process_manager.find_processes(&format!(":{}", port), FilterOptions::default());
    assert!(!results.is_empty());
    assert!(results.iter().all(|p| p.ports == Some(format!("{}", port))));
}

#[test]
fn should_find_cargo_process_by_pid() {
    let mut process_manager = ProcessManager::new().unwrap();
    let results = process_manager.find_processes("cargo", FilterOptions::default());
    let cargo_process_pid = results.nth(Some(0)).map(|r| r.pid).unwrap();

    let restults = process_manager
        .find_processes(&format!("!{}", cargo_process_pid), FilterOptions::default());
    assert_eq!(restults.len(), 1);
    assert_eq!(restults.nth(Some(0)).unwrap().pid, cargo_process_pid);
}

#[test]
fn should_find_cargo_process_by_process_family() {
    let mut process_manager = ProcessManager::new().unwrap();
    let results = process_manager.find_processes("cargo", FilterOptions::default());
    let cargo_process_pid = results.nth(Some(0)).map(|r| r.pid).unwrap();

    let results = process_manager
        .find_processes(&format!("@{}", cargo_process_pid), FilterOptions::default());
    assert!(!results.is_empty());
    assert!(results
        .iter()
        .all(|p| p.pid == cargo_process_pid || p.parent_pid == Some(cargo_process_pid)));
}

fn fuzzy_matches(value: &str, pattern: &str) -> bool {
    SkimMatcherV2::default()
        .fuzzy_match(value, pattern)
        .unwrap_or(0)
        > 0
}
