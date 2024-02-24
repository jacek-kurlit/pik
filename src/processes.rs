use anyhow::Result;
use sysinfo::{System, Users};

pub struct ProcessQuery {
    sys: System,
    users: Users,
}

impl ProcessQuery {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        //FIXME: Probably should only refresh processes info
        sys.refresh_all();
        let users = Users::new_with_refreshed_list();
        Self { sys, users }
    }

    pub fn find_processes(&self, query: &str) -> Result<Vec<Process>> {
        self.find_all_processes(query)
        // if query.is_empty() {
        //     return processes;
        // }
        // Ok(processes?
        //     .into_iter()
        //     .filter(|p| p.match_query(query))
        //     .collect::<Vec<Process>>())
    }

    fn find_all_processes(&self, query: &str) -> Result<Vec<Process>> {
        //sysinfo can search by name!
        let result = self
            .sys
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
                    //FIXME: maybe we should use Pid
                    pid: prc.pid().as_u32() as i32,
                    args: get_process_args(prc, &cmd),
                    cmd,
                    user_name,
                })
            })
            .collect();

        Ok(result)
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
    pub pid: i32,
    pub user_name: String,
    pub cmd: String,
    pub args: String,
}
