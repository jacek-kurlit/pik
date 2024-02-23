use anyhow::{Context, Result};
mod users;

use self::users::UserResolver;

pub struct ProcessQuery {
    user_resolver: UserResolver,
}

impl ProcessQuery {
    pub fn new() -> Self {
        Self {
            user_resolver: UserResolver::new(),
        }
    }

    pub fn find_processes(&self, query: &String) -> Result<Vec<Process>> {
        let processes = self.find_all_processes();
        if query.is_empty() {
            return processes;
        }
        Ok(processes?
            .into_iter()
            .filter(|p| p.match_query(query))
            .collect::<Vec<Process>>())
    }

    //FIXME: making this function filtering all errors make this more user friendly but hides the
    //error under the hood, user may be suprised when he gets empty list of processes, when
    //searching for specific process by name
    //on the other hand, some processes may quickly disappear, causing problems that we can get
    //process info
    fn find_all_processes(&self) -> Result<Vec<Process>> {
        let tps = procfs::ticks_per_second();
        let result = procfs::process::all_processes()
            .context("Could not load all processes")?
            .filter_map(|prc| {
                let prc = prc.ok()?;
                let user_id = prc.uid().ok()?;
                let stat = prc.stat().ok()?;

                // total_time is in seconds
                let total_time_seconds = (stat.utime + stat.stime) as f32 / (tps as f32);
                println!("{}", total_time_seconds);
                let user_name = self
                    .user_resolver
                    .resolve_name(user_id)
                    .unwrap_or("".to_string());
                Some(Process {
                    pid: stat.pid,
                    user_name,
                    args: get_process_args(&prc, &stat.comm),
                    cmd: stat.comm,
                })
            })
            .collect();

        Ok(result)
    }
}

// NOTE: Some processes have path to binary as first argument, but also some processes has different name than cmd (for exmaple firefox)
fn get_process_args(prc: &procfs::process::Process, cmd: &str) -> String {
    let args = prc.cmdline().unwrap_or_default();
    if args.first().is_some_and(|arg1| arg1.ends_with(cmd)) {
        return args[1..].join(", ");
    }
    args.join(", ")
}

pub struct Process {
    pub pid: i32,
    pub user_name: String,
    pub cmd: String,
    pub args: String,
}

impl Process {
    fn match_query(&self, query: &String) -> bool {
        self.cmd.contains(query) || self.args.contains(query)
    }
}
