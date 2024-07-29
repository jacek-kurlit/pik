use std::collections::HashMap;

use anyhow::Result;
use sysinfo::{Pid, System, Uid, Users};

mod query;
mod utils;

pub use query::SearchBy;

type ProcessPorts = HashMap<u32, Vec<String>>;

pub struct ProcessManager {
    sys: System,
    users: Users,
    process_ports: ProcessPorts,
    current_user_id: Uid,
}

#[derive(Copy, Clone)]
pub struct FilterOptions {
    //NOTE: On linux threads can be listed as processes and thus needs filtering
    pub ignore_threads: bool,
    pub user_processes_only: bool,
}

use query::ProcessFilter;

use self::utils::{
    find_current_process_user, format_as_epoch_time, format_as_hh_mm_ss, get_process_args,
};

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

    pub fn remove(&mut self, index: Option<usize>) -> Option<Process> {
        let index = index?;
        Some(self.items.remove(index))
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
        let process_filter = ProcessFilter::new(query);

        let items = self
            .sys
            .processes()
            .values()
            .filter(|prc| {
                if options.ignore_threads && prc.thread_kind().is_some() {
                    return false;
                }
                !options.user_processes_only || prc.user_id() == Some(&self.current_user_id)
            })
            .map(|prc| self.create_process_info(prc))
            .filter(|prc| process_filter.apply(prc))
            .collect();

        ProcessSearchResults {
            search_by: process_filter.search_by,
            items,
        }
    }

    fn create_process_info(&self, prc: &sysinfo::Process) -> Process {
        let user_name = prc
            .user_id()
            .map(|user_id| {
                self.users
                    .get_user_by_id(user_id)
                    .map(|u| u.name().to_string())
                    .unwrap_or(format!("{}?", **user_id))
            })
            .unwrap_or("unknown".to_string());
        let cmd = prc.name().to_string();
        let cmd_path = prc.exe().map(|e| e.to_string_lossy().to_string());
        let pid = prc.pid().as_u32();
        let ports = self.process_ports.get(&pid).map(|ports| ports.join(","));

        Process {
            pid,
            parent_pid: prc.parent().map(|p| p.as_u32()),
            args: get_process_args(prc, &cmd_path, &cmd),
            cmd,
            cmd_path,
            user_name,
            ports,
            memory: prc.memory(),
            start_time: format_as_epoch_time(prc.start_time()),
            run_time: format_as_hh_mm_ss(prc.run_time()),
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

fn refresh_ports() -> HashMap<u32, Vec<String>> {
    listeners::get_all()
        //NOTE: we ignore errors comming from listeners
        .unwrap_or_default()
        .into_iter()
        .fold(HashMap::new(), |mut acc: ProcessPorts, l| {
            acc.entry(l.process.pid)
                .or_default()
                .push(l.socket.port().to_string());
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
