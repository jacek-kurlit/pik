use std::{borrow::Cow, collections::HashMap};

use anyhow::Result;
use sysinfo::{Pid, System, Uid, Users};

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
            .filter(|prc| process_filter.apply(*prc, &self.process_ports))
            .map(|prc| self.create_process_info(prc))
            .collect()
    }

    fn create_process_info(&self, prc: &impl ProcessInfo) -> Process {
        let user_name = prc
            .user_id()
            .and_then(|user_id| {
                self.users
                    .get_user_by_id(user_id)
                    .map(|u| u.name().to_string())
            })
            .unwrap_or("".to_string());
        let cmd = prc.cmd().to_string();
        let exe_path = prc
            .exe_path()
            .map(|e| e.to_string())
            .unwrap_or("".to_string());
        let ports = self
            .process_ports
            .get(&prc.pid_u32())
            .map(|ports| ports.join(","))
            .unwrap_or_default();
        Process {
            pid: prc.pid_u32(),
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
fn get_process_args(prc: &impl ProcessInfo, exe_path: &str, cmd: &str) -> String {
    let args = prc.args();
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

//TODO: use this instead of sysinfo::Process
// this probably could be also ProcessInfo<sysinfo::Process>
pub trait ProcessInfo {
    fn pid_u32(&self) -> u32;

    fn user_id(&self) -> Option<&Uid>;

    fn is_thread(&self) -> bool;

    fn cmd(&self) -> &str;

    fn exe_path(&self) -> Option<Cow<str>>;

    fn args(&self) -> &[String];
}

impl ProcessInfo for sysinfo::Process {
    fn is_thread(&self) -> bool {
        self.thread_kind().is_some()
    }

    fn cmd(&self) -> &str {
        self.name()
    }

    fn exe_path(&self) -> Option<Cow<str>> {
        self.exe().map(|exe| exe.to_string_lossy())
    }

    //FIXME: this ay cause bug because we are filtering out first argument which most of the time
    //is exe path
    fn args(&self) -> &[String] {
        self.cmd()
    }

    fn pid_u32(&self) -> u32 {
        self.pid().as_u32()
    }

    fn user_id(&self) -> Option<&Uid> {
        self.user_id()
    }
}
