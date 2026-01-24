use std::cmp::Ordering;
use std::time::SystemTime;

use anyhow::{Ok, Result};
use sysinfo::{Pid, SUPPORTED_SIGNALS, System, Uid, Users};
use sysinfo::{ProcessRefreshKind, Signal};

mod daemon;
mod filters;
mod ports;
mod utils;

pub use daemon::*;
pub use filters::IgnoreOptions;
pub use filters::SearchBy;

use filters::QueryFilter;

#[cfg_attr(test, faux::create)]
pub struct ProcessManager {
    sys: System,
    users: Users,
    process_ports: ProcessPorts,
    current_user_id: Uid,
}

use crate::processes::ports::ProcessPorts;

use self::filters::IgnoreProcessesFilter;
use self::utils::{
    find_current_process_user, get_process_args, process_run_time, to_system_local_time,
};

pub trait ProcessInfo {
    fn is_thread(&self) -> bool;

    fn user_id(&self) -> Option<&Uid>;

    fn cmd(&self) -> &str;

    fn cmd_path(&self) -> Option<&str>;

    fn pid(&self) -> u32;

    fn parent_id(&self) -> Option<u32>;

    fn memory(&self) -> u64;

    fn start_time(&self) -> u64;

    fn run_time(&self) -> u64;

    fn args(&self) -> Vec<&str>;
}

impl ProcessInfo for sysinfo::Process {
    fn is_thread(&self) -> bool {
        self.thread_kind().is_some()
    }

    fn user_id(&self) -> Option<&Uid> {
        self.user_id()
    }

    fn cmd(&self) -> &str {
        self.name().to_str().unwrap_or_default()
    }

    fn cmd_path(&self) -> Option<&str> {
        self.exe().map(|e| e.to_str()).unwrap_or_default()
    }

    fn pid(&self) -> u32 {
        self.pid().as_u32()
    }

    fn parent_id(&self) -> Option<u32> {
        self.parent().map(|p| p.as_u32())
    }

    fn memory(&self) -> u64 {
        self.memory()
    }

    fn start_time(&self) -> u64 {
        self.start_time()
    }

    fn run_time(&self) -> u64 {
        self.start_time()
    }

    fn args(&self) -> Vec<&str> {
        self.cmd().iter().filter_map(|a| a.to_str()).collect()
    }
}

#[derive(Debug)]
pub struct ProcessSearchResults {
    pub items: Vec<ResultItem>,
}

impl ProcessSearchResults {
    pub fn empty() -> Self {
        Self { items: vec![] }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn nth(&self, index: Option<usize>) -> Option<&Process> {
        let index = index?;
        self.items.get(index).map(|item| &item.process)
    }

    pub fn remove(&mut self, pid: u32) {
        self.items.retain(|item| item.process.pid != pid)
    }

    pub fn iter(&self) -> impl Iterator<Item = &ResultItem> {
        self.items.iter()
    }
}

#[cfg_attr(test, faux::methods)]
impl ProcessManager {
    pub fn new() -> Result<Self> {
        let mut sys = System::new();
        let mut users = Users::new_with_refreshed_list();
        let process_ports = optimized_refresh(&mut sys, &mut users);
        let current_user_id = find_current_process_user(&sys)?;

        Ok(Self {
            sys,
            users,
            process_ports,
            current_user_id,
        })
    }

    pub fn find_processes(&mut self, query: &str, ignore: &IgnoreOptions) -> ProcessSearchResults {
        let query_filter = QueryFilter::new(query);
        let ignored_processes_filter = IgnoreProcessesFilter::new(ignore, &self.current_user_id);

        let mut items = self
            .sys
            .processes()
            .values()
            .filter(|prc| ignored_processes_filter.accept(*prc))
            .filter_map(|prc| {
                let ports = self.process_ports.get(&prc.pid().as_u32());
                let match_data = query_filter.accept(prc, ports.map(|p| p.as_str()))?;
                Some(ResultItem::new(
                    match_data,
                    self.create_process_info(prc, ports),
                ))
            })
            .collect::<Vec<ResultItem>>();

        items.sort_by(|a, b| a.match_data.match_type.cmp(&b.match_data.match_type));

        ProcessSearchResults { items }
    }

    pub fn refresh(&mut self) {
        self.process_ports = optimized_refresh(&mut self.sys, &mut self.users);
    }

    fn create_process_info(&self, prc: &impl ProcessInfo, ports: Option<&String>) -> Process {
        let user_name = prc
            .user_id()
            .map(|user_id| {
                self.users
                    .get_user_by_id(user_id)
                    .map(|u| u.name().to_string())
                    .unwrap_or(format!("{}?", **user_id))
            })
            .unwrap_or("unknown".to_string());
        let cmd = prc.cmd().to_string();
        let cmd_path = prc.cmd_path().map(|p| p.to_string());
        let pid = prc.pid();

        Process {
            pid,
            parent_pid: prc.parent_id(),
            args: get_process_args(prc).unwrap_or_default(),
            cmd,
            cmd_path,
            user_name,
            ports: ports.cloned(),
            memory: prc.memory(),
            start_time: to_system_local_time(prc.start_time())
                .format("%H:%M:%S")
                .to_string(),
            run_time: process_run_time(prc.run_time(), SystemTime::now()),
        }
    }

    pub fn kill_process(&self, pid: u32, graceful: bool) -> bool {
        match self.sys.process(Pid::from_u32(pid)) {
            Some(prc) => {
                let signal = determine_kill_signal(graceful);
                prc.kill_with(signal).unwrap_or(false)
            }
            None => false,
        }
    }
}

fn determine_kill_signal(graceful: bool) -> Signal {
    //windows does not support graceful kill
    if graceful && SUPPORTED_SIGNALS.contains(&Signal::Term) {
        Signal::Term
    } else {
        Signal::Kill
    }
}

fn process_refresh_kind() -> ProcessRefreshKind {
    ProcessRefreshKind::default()
        .with_cpu()
        .with_memory()
        .with_cmd(sysinfo::UpdateKind::OnlyIfNotSet)
        .with_exe(sysinfo::UpdateKind::OnlyIfNotSet)
        .with_user(sysinfo::UpdateKind::OnlyIfNotSet)
}

#[derive(Debug, Clone)]
pub struct Process {
    pub pid: u32,
    pub parent_pid: Option<u32>,
    pub user_name: String,
    pub cmd: String,
    pub cmd_path: Option<String>,
    pub args: String,
    pub ports: Option<String>,
    pub memory: u64,
    //FIXME: cpu rquires refresh twice!
    // pub cpu_usage: f32,
    pub start_time: String,
    pub run_time: String,
}

impl Process {
    pub fn exe(&self) -> &str {
        self.cmd_path.as_ref().unwrap_or(&self.cmd)
    }

    pub fn parent_as_string(&self) -> String {
        self.parent_pid
            .map(|pid| pid.to_string())
            .unwrap_or_default()
    }
}

#[derive(Debug)]
pub struct ResultItem {
    pub match_data: MatchData,
    pub process: Process,
}

impl ResultItem {
    pub fn new(match_data: MatchData, process: Process) -> Self {
        Self {
            match_data,
            process,
        }
    }

    pub fn is_matched_by(&self, matched_by: MatchedBy) -> bool {
        self.match_data.matched_by == matched_by
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct MatchData {
    pub matched_by: MatchedBy,
    pub match_type: MatchType,
}

impl MatchData {
    pub fn new(matched_by: MatchedBy, match_type: MatchType) -> Self {
        Self {
            matched_by,
            match_type,
        }
    }
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum MatchedBy {
    Cmd,
    Args,
    Path,
    Port,
    Pid,
    ParentPid,
    ProcessExistence,
}

#[derive(PartialEq, Eq, Debug)]

pub enum MatchType {
    Exact,
    Fuzzy { score: i64, positions: Vec<usize> },
    Exists,
}

impl PartialOrd for MatchType {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// This is needed as we sort by match type. Exact matches should go first, Exists should go last
/// and fuzzy matches should be sorted by score
impl Ord for MatchType {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (MatchType::Exact, MatchType::Exact) => Ordering::Equal,
            (MatchType::Exact, _) => Ordering::Less,
            (_, MatchType::Exact) => Ordering::Greater,

            (MatchType::Fuzzy { score: s1, .. }, MatchType::Fuzzy { score: s2, .. }) => s2.cmp(s1),
            (MatchType::Fuzzy { .. }, _) => Ordering::Less,
            (_, MatchType::Fuzzy { .. }) => Ordering::Greater,

            (MatchType::Exists, MatchType::Exists) => Ordering::Equal,
        }
    }
}

/// Refreshes the system information, including processes and their associated ports.
/// This method spawns a separate thread to refresh the ports, as it speeds up the overall refresh process.
/// It makes overall refreshes ~2x faster (initial refresh is slower though).
fn optimized_refresh(sys: &mut System, users: &mut Users) -> ProcessPorts {
    let ports_refresh = std::thread::spawn(ProcessPorts::new_refreshed);
    sys.refresh_processes_specifics(
        sysinfo::ProcessesToUpdate::All,
        true,
        process_refresh_kind(),
    );

    users.refresh();
    ports_refresh.join().unwrap_or_default()
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn match_type_sort_in_correct_order() {
        let mut vec_to_sort = vec![
            MatchType::Exists,
            MatchType::Fuzzy {
                score: 1,
                positions: vec![10, 20],
            },
            MatchType::Fuzzy {
                score: 1,
                positions: vec![30, 40],
            },
            MatchType::Fuzzy {
                score: 10,
                positions: vec![1, 2],
            },
            MatchType::Exact,
        ];
        vec_to_sort.sort();
        assert_eq!(
            vec_to_sort,
            vec![
                MatchType::Exact,
                MatchType::Fuzzy {
                    score: 10,
                    positions: vec![1, 2]
                },
                MatchType::Fuzzy {
                    score: 1,
                    positions: vec![10, 20]
                },
                MatchType::Fuzzy {
                    score: 1,
                    positions: vec![30, 40]
                },
                MatchType::Exists,
            ]
        );
    }

    #[cfg(target_family = "unix")]
    #[test]
    fn should_determine_correct_kill_signal_for_unix() {
        assert_eq!(determine_kill_signal(true), Signal::Term);
        assert_eq!(determine_kill_signal(false), Signal::Kill);
    }

    #[cfg(target_family = "windows")]
    #[test]
    fn should_determine_correct_kill_signal_for_windows() {
        assert_eq!(determine_kill_signal(true), Signal::Kill);
        assert_eq!(determine_kill_signal(false), Signal::Kill);
    }
}
