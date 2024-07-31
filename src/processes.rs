use std::collections::HashMap;

use anyhow::Result;
use sysinfo::ProcessRefreshKind;
use sysinfo::{Pid, System, Uid, Users};

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
    find_current_process_user, format_as_epoch_time, format_seconds_as_hh_mm_ss, get_process_args,
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

    fn args(&self) -> &[String];
}

impl ProcessInfo for sysinfo::Process {
    fn is_thread(&self) -> bool {
        self.thread_kind().is_some()
    }

    fn user_id(&self) -> Option<&Uid> {
        self.user_id()
    }

    fn cmd(&self) -> &str {
        self.name()
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

    fn args(&self) -> &[String] {
        self.cmd()
    }
}

pub struct ProcessSearchResults {
    pub search_by: SearchBy,
    items: Vec<Process>,
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

    pub fn nth(&self, index: Option<usize>) -> Option<&Process> {
        let index = index?;
        self.items.get(index)
    }

    pub fn remove(&mut self, pid: u32) {
        self.items.retain(|prc| prc.pid != pid)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Process> {
        self.items.iter()
    }
}

impl ProcessManager {
    pub fn new() -> Result<Self> {
        //TODO: maybe we should not refresh all informaton since they are not needed, just the one
        //we need
        let sys = System::new_all();
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

        let items = self
            .sys
            .processes()
            .values()
            .filter_map(|prc| {
                let ports = self.process_ports.get(&prc.pid().as_u32());
                if !options_filter.accept(prc)
                    || !process_filter.accept(prc, ports.map(|p| p.as_str()))
                {
                    return None;
                }
                Some(self.create_process_info(prc, ports))
            })
            .collect();

        ProcessSearchResults {
            search_by: process_filter.search_by,
            items,
        }
    }

    pub fn refresh(&mut self) {
        self.sys
            .refresh_processes_specifics(ProcessRefreshKind::everything());
        //TODO: do we really need to refresh users?
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
            args: get_process_args(prc, &cmd_path, &cmd),
            cmd,
            cmd_path,
            user_name,
            ports: ports.cloned(),
            memory: prc.memory(),
            start_time: format_as_epoch_time(prc.start_time()),
            run_time: format_seconds_as_hh_mm_ss(prc.run_time()),
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

fn refresh_ports() -> HashMap<u32, String> {
    listeners::get_all()
        //NOTE: we ignore errors comming from listeners
        .unwrap_or_default()
        .into_iter()
        .fold(HashMap::new(), |mut acc: ProcessPorts, l| {
            acc.entry(l.process.pid)
                .or_default()
                .push_str(&format!("{}, ", l.socket.port()));
            acc
        })
}

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
