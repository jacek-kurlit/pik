use std::collections::HashMap;

use anyhow::Result;
use sysinfo::{Pid, System, Users};

mod query;

type ProcessPorts = HashMap<u32, Vec<String>>;

pub struct ProcessManager {
    sys: System,
    users: Users,
    process_ports: ProcessPorts,
}

use query::ProcessFilter;

impl ProcessManager {
    pub fn new() -> Result<Self> {
        let sys = System::new_all();
        let users = Users::new_with_refreshed_list();
        let process_ports = listeners::get_all()
            .map_err(|e| anyhow::anyhow!("Failed to get listeners: {e}"))?
            .into_iter()
            .fold(HashMap::new(), |mut acc: ProcessPorts, l| {
                acc.entry(l.process.pid)
                    .or_default()
                    .push(l.socket.port().to_string());
                acc
            });
        Ok(Self {
            sys,
            users,
            process_ports,
        })
    }

    pub fn find_processes(&mut self, query: &str) -> Vec<Process> {
        let process_filter = ProcessFilter::new(query);

        self.sys
            .processes()
            .values()
            //NOTE: On linux threads can be listed as processes and thus needs filtering
            .filter(|prc| prc.thread_kind().is_none())
            .map(|prc| self.create_process_info(prc))
            .filter(|prc| process_filter.apply(prc))
            .collect()
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
        let exe_path = prc
            .exe()
            .map(|e| e.to_string_lossy().to_string())
            .unwrap_or("".to_string());
        let pid = prc.pid().as_u32();
        let ports = self
            .process_ports
            .get(&pid)
            .map(|ports| ports.join(","))
            .unwrap_or_default();
        Process {
            pid,
            args: get_process_args(prc, &exe_path, &cmd),
            cmd,
            exe_path,
            user_name,
            ports,
        }
    }

    pub fn kill_process(&self, pid: u32) {
        if let Some(prc) = self.sys.process(Pid::from_u32(pid)) {
            if sysinfo::SUPPORTED_SIGNALS.contains(&sysinfo::Signal::Term) {
                prc.kill_with(sysinfo::Signal::Term);
            } else {
                prc.kill();
            }
        }
    }
}

// NOTE: Some processes have path to binary as first argument, but also some processes has different name than cmd (for exmaple firefox)
fn get_process_args(prc: &sysinfo::Process, exe_path: &str, cmd: &str) -> String {
    let args = prc.cmd();
    if args
        .first()
        .is_some_and(|arg1| arg1 == exe_path || arg1.ends_with(cmd))
    {
        return args[1..].join(", ");
    }
    args.join(", ")
}

pub struct Process {
    pub pid: u32,
    pub user_name: String,
    pub cmd: String,
    pub exe_path: String,
    pub args: String,
    pub ports: String,
}
