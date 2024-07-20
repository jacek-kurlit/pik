use std::collections::HashMap;

use anyhow::Result;
use sysinfo::{Pid, System, Users};

mod query;
mod utils;

pub use query::SearchBy;

type ProcessPorts = HashMap<u32, Vec<String>>;

pub struct ProcessManager {
    sys: System,
    users: Users,
    process_ports: ProcessPorts,
}

use query::ProcessFilter;

use self::utils::{format_as_epoch_time, format_as_hh_mm_ss, get_process_args};

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

    pub fn iter(&self) -> impl Iterator<Item = &Process> {
        self.items.iter()
    }
}

impl ProcessManager {
    pub fn new() -> Result<Self> {
        let sys = System::new();
        let users = Users::new();
        let process_ports = HashMap::new();
        let mut manager = Self {
            sys,
            users,
            process_ports,
        };
        manager.refresh();
        Ok(manager)
    }

    pub fn kill_and_refresh(&mut self, pid: u32, query: &str) -> ProcessSearchResults {
        self.kill_process(pid);
        self.refresh();
        let mut search_results = self.find_processes(query);
        //FIXME: on linux t takes time for the process to be killed and refresh may still find it!
        //this should be fixed if we implement autorefresh
        search_results.items.retain(|prc| prc.pid != pid);
        search_results
    }

    fn refresh(&mut self) {
        //TODO: maybe we should not refresh all informaton since they are not needed, just the one
        //we need
        self.sys.refresh_all();
        self.users.refresh_list();
        self.process_ports = refresh_ports();
    }

    pub fn find_processes(&mut self, query: &str) -> ProcessSearchResults {
        let process_filter = ProcessFilter::new(query);

        let items = self
            .sys
            .processes()
            .values()
            //NOTE: On linux threads can be listed as processes and thus needs filtering
            .filter(|prc| prc.thread_kind().is_none())
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
            .and_then(|user_id| {
                self.users
                    .get_user_by_id(user_id)
                    .map(|u| u.name().to_string())
            })
            .unwrap_or("".to_string());
        let cmd = prc.name().to_string();
        let exe_path = prc.exe().map(|e| e.to_string_lossy().to_string());
        let pid = prc.pid().as_u32();
        let ports = self.process_ports.get(&pid).map(|ports| ports.join(","));

        Process {
            pid,
            parent_pid: prc.parent().map(|p| p.as_u32()),
            args: get_process_args(prc, &exe_path, &cmd),
            cmd,
            cmd_path: exe_path,
            user_name,
            ports,
            memory: prc.memory(),
            start_time: format_as_epoch_time(prc.start_time()),
            run_time: format_as_hh_mm_ss(prc.run_time()),
        }
    }

    fn kill_process(&self, pid: u32) {
        if let Some(prc) = self.sys.process(Pid::from_u32(pid)) {
            if sysinfo::SUPPORTED_SIGNALS.contains(&sysinfo::Signal::Term) {
                //FIXME: add handling for success / failure!
                prc.kill_with(sysinfo::Signal::Term);
            } else {
                prc.kill();
            }
        }
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
