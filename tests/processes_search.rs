use std::{thread, time::Duration};

use pik::processes::{IgnoreOptions, ProcessManager, ProcessSearchResults};

use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};

#[test]
fn should_find_cargo_process_by_cmd_name() {
    let mut process_manager = ProcessManager::new().unwrap();
    let results = process_manager.find_processes("cargo", &IgnoreOptions::default());
    assert!(!results.is_empty());
    assert!(
        results
            .iter()
            .all(|item| fuzzy_matches(&item.process.cmd, "cargo"))
    );
    assert!(results_are_sorted_by_match_type(results));
}

#[test]
fn should_find_cargo_process_by_cmd_path() {
    let mut process_manager = ProcessManager::new().unwrap();
    let results = process_manager.find_processes("/cargo", &IgnoreOptions::default());
    assert!(!results.is_empty());
    assert!(
        results
            .iter()
            .all(|item| fuzzy_matches(item.process.cmd_path.as_ref().unwrap(), "cargo"))
    );
    assert!(results_are_sorted_by_match_type(results));
}

#[test]
fn should_find_cargo_process_by_name_path_or_args() {
    let mut process_manager = ProcessManager::new().unwrap();
    let results = process_manager.find_processes("~cargo", &IgnoreOptions::default());
    assert!(!results.is_empty());
    assert!(results.iter().all(|item| fuzzy_matches(
        item.process.cmd_path.as_ref().unwrap(),
        "cargo"
    ) || fuzzy_matches(&item.process.args, "cargo")
        || fuzzy_matches(&item.process.cmd, "cargo")));
    assert!(results_are_sorted_by_match_type(results));
}

#[test]
fn should_find_cargo_process_by_args() {
    let mut process_manager = ProcessManager::new().unwrap();
    let results = process_manager.find_processes("-test", &IgnoreOptions::default());
    assert!(!results.is_empty());
    assert!(
        results
            .iter()
            .all(|item| fuzzy_matches(&item.process.args, "test"))
    );
    assert!(results_are_sorted_by_match_type(results));
}

use http_test_server::TestServer;
#[test]
fn should_find_cargo_process_by_port() {
    let test_server = TestServer::new().unwrap();
    let port = test_server.port();
    // NOTE: Sometimes system needs time to notice the port is in use
    thread::sleep(Duration::from_millis(250));
    let mut process_manager = ProcessManager::new().unwrap();
    let results = process_manager.find_processes(&format!(":{}", port), &IgnoreOptions::default());
    assert!(!results.is_empty());
    assert!(results.iter().all(|item| {
        fuzzy_matches(
            item.process.ports.as_deref().unwrap_or(""),
            &format!("{}", port),
        )
    }));
    assert!(results_are_sorted_by_match_type(results));
}

#[test]
fn should_find_cargo_process_by_pid() {
    let mut process_manager = ProcessManager::new().unwrap();
    let results = process_manager.find_processes("cargo", &IgnoreOptions::default());
    let cargo_process_pid = results.nth(Some(0)).map(|r| r.pid).unwrap();

    let restults = process_manager.find_processes(
        &format!("!{}", cargo_process_pid),
        &IgnoreOptions::default(),
    );
    assert_eq!(restults.len(), 1);
    assert_eq!(restults.nth(Some(0)).unwrap().pid, cargo_process_pid);
    assert!(results_are_sorted_by_match_type(results));
}

#[test]
fn should_find_cargo_process_by_process_family() {
    let mut process_manager = ProcessManager::new().unwrap();
    let results = process_manager.find_processes("cargo", &IgnoreOptions::default());
    let cargo_process_pid = results.nth(Some(0)).map(|r| r.pid).unwrap();

    let results = process_manager.find_processes(
        &format!("@{}", cargo_process_pid),
        &IgnoreOptions::default(),
    );
    assert!(!results.is_empty());
    assert!(
        results
            .iter()
            .all(|item| item.process.pid == cargo_process_pid
                || item.process.parent_pid == Some(cargo_process_pid))
    );
    assert!(results_are_sorted_by_match_type(results));
}

#[cfg(target_family = "unix")]
#[test]
fn should_ignore_processes_in_usr_dir() {
    use regex::Regex;

    let mut process_manager = ProcessManager::new().unwrap();
    let ignore = IgnoreOptions {
        paths: vec![Regex::new("/usr/*").unwrap()],
        ..Default::default()
    };
    let results = process_manager.find_processes("", &ignore);
    assert!(!results.is_empty());
    assert!(results.iter().all(|item| {
        item.process
            .cmd_path
            .as_ref()
            .map(|path| !path.starts_with("/usr"))
            .unwrap_or(true)
    }));
}

fn fuzzy_matches(value: &str, pattern: &str) -> bool {
    let (_, indicies) = SkimMatcherV2::default()
        .fuzzy_indices(value, pattern)
        .unwrap_or((0, vec![]));
    !indicies.is_empty()
}

fn results_are_sorted_by_match_type(results: ProcessSearchResults) -> bool {
    results
        .items
        .iter()
        .zip(results.items.iter().skip(1))
        .all(|(a, b)| a.match_data.match_type <= b.match_data.match_type)
}
