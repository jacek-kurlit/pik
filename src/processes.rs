use anyhow::{anyhow, Context, Result};
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

    pub fn find_all_processes(&self) -> Result<Vec<Process>> {
        let mut result = Vec::new();
        let tps = procfs::ticks_per_second();

        println!("{: >10} {: <8} {: >8} CMD", "PID", "TTY", "TIME");

        for prc in procfs::process::all_processes().context("Could not load all processes")? {
            let prc = prc?;
            if let Ok(stat) = prc.stat() {
                // total_time is in seconds
                let total_time = (stat.utime + stat.stime) as f32 / (tps as f32);
                let user_name = self
                    .user_resolver
                    .resolve_name(prc.uid()?)
                    .ok_or(anyhow!("Could not find user of process {}", stat.pid))?;
                result.push(Process {
                    pid: stat.pid,
                    user_name,
                    total_time,
                    cmd: stat.comm,
                });
            }
        }
        Ok(result)
    }
}

pub struct Process {
    pub pid: i32,
    pub user_name: String,
    pub total_time: f32,
    pub cmd: String,
}

impl Process {
    pub fn ref_array(&self) -> [String; 4] {
        [
            format!("\n{}\n", self.user_name),
            format!("\n{}\n", self.pid),
            format!("\n{}\n", self.total_time),
            format!("\n{}\n", self.cmd),
        ]
    }
}
