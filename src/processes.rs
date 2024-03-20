use sysinfo::{Pid, System, Users};

pub struct ProcessManager {
    sys: System,
    users: Users,
}

impl ProcessManager {
    pub fn new() -> Self {
        let sys = System::new_all();
        let users = Users::new_with_refreshed_list();
        Self { sys, users }
    }

    pub fn find_processes(&mut self, query: &str) -> Vec<Process> {
        self.sys.refresh_processes();
        self.sys
            .processes_by_name(query)
            .filter_map(|prc| {
                //NOTE: On linux threads can be listed as processes and thus needs filtering
                if prc.thread_kind().is_some() {
                    return None;
                }
                let user_id = prc.user_id()?;
                let user_name = self
                    .users
                    .get_user_by_id(user_id)
                    .map(|u| u.name().to_string())
                    .unwrap_or("".to_string());
                let cmd = prc.name().to_string();
                Some(Process {
                    pid: prc.pid().as_u32(),
                    args: get_process_args(prc, &cmd),
                    cmd,
                    user_name,
                })
            })
            .collect()
    }

    pub fn kill_process(&self, pid: u32) {
        if let Some(prc) = self.sys.process(Pid::from_u32(pid)) {
            prc.kill();
        }
    }
}

// NOTE: Some processes have path to binary as first argument, but also some processes has different name than cmd (for exmaple firefox)
fn get_process_args(prc: &sysinfo::Process, cmd: &str) -> String {
    let args = prc.cmd();
    if args.first().is_some_and(|arg1| arg1.ends_with(cmd)) {
        return args[1..].join(", ");
    }
    args.join(", ")
}

//TODO: consider if this is even needed
pub struct Process {
    pub pid: u32,
    pub user_name: String,
    pub cmd: String,
    pub args: String,
}
