use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use procfs::process::ProcState;
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

    //FIXME: failing entire app because of single error during fetch of indiwidual status may be not best CX
    pub fn find_all_processes(&self) -> Result<Vec<Process>> {
        let mut result = Vec::new();
        let tps = procfs::ticks_per_second();

        for prc in procfs::process::all_processes().context("Could not load all processes")? {
            let prc = prc?;
            if let Ok(stat) = prc.stat() {
                // total_time is in seconds
                let total_time = (stat.utime + stat.stime) as f32 / (tps as f32);
                let state = stat
                    .state()
                    .context(format!("Failed to retrieve process {} status", stat.pid))?;

                let user_name = self
                    .user_resolver
                    .resolve_name(prc.uid()?)
                    .ok_or(anyhow!("Could not find user of process {}", stat.pid))?;
                result.push(Process {
                    pid: stat.pid,
                    user_name,
                    total_time: Duration::from_secs_f32(total_time),
                    args: get_process_args(&prc, &stat.comm),
                    cmd: stat.comm,
                    state,
                });
            }
        }
        Ok(result)
    }
}

// NOTE: Some processes have path to binary as first argument, but also some processes has different name than cmd (for exmaple firefox)
fn get_process_args(prc: &procfs::process::Process, cmd: &str) -> Vec<String> {
    let args = prc.cmdline().unwrap_or_default();
    if args.first().is_some_and(|arg1| arg1.ends_with(cmd)) {
        return args.into_iter().skip(1).collect();
    }
    args
}

pub struct Process {
    pub pid: i32,
    pub user_name: String,
    pub total_time: Duration,
    pub cmd: String,
    pub state: ProcState,
    pub args: Vec<String>,
}
