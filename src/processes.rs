use std::collections::HashMap;
use std::time::SystemTime;

use anyhow::Result;
use sysinfo::{Pid, System, Uid, Users};
use sysinfo::{ProcessRefreshKind, RefreshKind};

mod filters;
mod utils;

pub use filters::FilterOptions;
pub use filters::SearchBy;

use filters::QueryFilter;

pub type ProcessPorts = HashMap<u32, String>;

pub struct ProcessManager {
    sys: System,
    users: Users,
    process_ports: ProcessPorts,
    current_user_id: Uid,
}

use self::filters::OptionsFilter;
use self::utils::{
    find_current_process_user, get_process_args, process_run_time, process_start_time,
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
    pub search_by: SearchBy,
    pub items: Vec<ResultItem>,
}

impl ProcessSearchResults {
    pub fn empty() -> Self {
        Self {
            search_by: SearchBy::None,
            items: vec![],
        }
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

    pub fn iter(&self) -> impl Iterator<Item = &Process> {
        self.items.iter().map(|i| &i.process)
    }
}

impl ProcessManager {
    pub fn new() -> Result<Self> {
        let sys = System::new_with_specifics(
            RefreshKind::default().with_processes(process_refresh_kind()),
        );
        let users = Users::new_with_refreshed_list();
        let process_ports = refresh_ports();
        let current_user_id = find_current_process_user(&sys)?;
        Ok(Self {
            sys,
            users,
            process_ports,
            current_user_id,
        })
    }

    pub fn find_processes(&mut self, query: &str, options: FilterOptions) -> ProcessSearchResults {
        let process_filter = QueryFilter::new(query);
        let options_filter = OptionsFilter::new(options, &self.current_user_id);

        let mut items = self
            .sys
            .processes()
            .values()
            .filter_map(|prc| {
                let ports = self.process_ports.get(&prc.pid().as_u32());
                if !options_filter.accept(prc) {
                    return None;
                }

                let match_data = process_filter.accept(prc, ports.map(|p| p.as_str()));
                if match_data.negative_match() {
                    return None;
                }
                Some(ResultItem::new(
                    match_data,
                    self.create_process_info(prc, ports),
                ))
            })
            .collect::<Vec<ResultItem>>();

        items.sort_by(|a, b| b.match_data.score.cmp(&a.match_data.score));

        ProcessSearchResults {
            search_by: process_filter.search_by,
            items,
        }
    }

    pub fn refresh(&mut self) {
        self.sys.refresh_processes_specifics(
            sysinfo::ProcessesToUpdate::All,
            true,
            process_refresh_kind(),
        );
        // TODO: do we really need to refresh users?
        self.users.refresh_list();
        self.process_ports = refresh_ports();
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
            args: get_process_args(prc).join(",").to_string(),
            cmd,
            cmd_path,
            user_name,
            ports: ports.cloned(),
            memory: prc.memory(),
            start_time: process_start_time(prc.start_time()),
            run_time: process_run_time(prc.run_time(), SystemTime::now()),
        }
    }

    pub fn kill_process(&self, pid: u32) -> bool {
        return match self.sys.process(Pid::from_u32(pid)) {
            Some(prc) => {
                if sysinfo::SUPPORTED_SIGNALS.contains(&sysinfo::Signal::Term) {
                    prc.kill_with(sysinfo::Signal::Term).unwrap_or(false)
                } else {
                    prc.kill()
                }
            }
            None => false,
        };
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

fn refresh_ports() -> HashMap<u32, String> {
    listeners::get_all()
        //NOTE: we ignore errors comming from listeners
        .unwrap_or_default()
        .into_iter()
        .fold(HashMap::new(), |mut acc: ProcessPorts, l| {
            match acc.get_mut(&l.process.pid) {
                Some(ports) => {
                    ports.push_str(&format!(", {}", l.socket.port()));
                }
                None => {
                    acc.insert(l.process.pid, format!("{}", l.socket.port()));
                }
            }
            acc
        })
}

#[derive(Debug)]
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
}

#[derive(PartialEq, Eq, Debug)]
pub struct MatchData {
    pub score: i64,
}

impl MatchData {
    pub fn new(score: i64) -> Self {
        Self { score }
    }

    pub fn perfect() -> Self {
        Self { score: i64::MAX }
    }

    pub fn none() -> Self {
        Self { score: -1 }
    }

    pub fn positive_match(&self) -> bool {
        self.score > 0
    }

    pub fn negative_match(&self) -> bool {
        self.score <= 0
    }
}
