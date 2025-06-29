use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, anyhow};
use chrono::{DateTime, Local};
use itertools::Itertools;
use sysinfo::{System, Uid};

use super::ProcessInfo;

// NOTE: Some processes have path to binary as first argument, but also some processes has different name than cmd (for example Firefox)
pub(super) fn get_process_args(prc: &impl ProcessInfo) -> Option<String> {
    let args = prc.args();
    let cmd_path = prc.cmd_path().unwrap_or("");
    let skip_first = match args.first() {
        Some(first_arg) if *first_arg == cmd_path || first_arg.ends_with(prc.cmd()) => 1,
        _ => 0,
    };
    let joined_args = args.into_iter().skip(skip_first).join(" ");
    match joined_args.trim().is_empty() {
        true => None,
        false => Some(joined_args),
    }
}

pub(super) fn process_run_time(run_duration_since_epoch: u64, now: SystemTime) -> String {
    let now_since_epoch = now.duration_since(UNIX_EPOCH).unwrap().as_secs();
    let seconds_diff = now_since_epoch.saturating_sub(run_duration_since_epoch);
    let seconds = seconds_diff % 60;
    let hours = seconds_diff / 3600;
    let minutes = (seconds_diff % 3600) / 60;
    if hours > 0 {
        return format!("{hours}h {minutes}m {seconds}s");
    }
    if minutes > 0 {
        return format!("{minutes}m {seconds}s");
    }
    format!("{seconds}s")
}

pub(super) fn to_system_local_time(seconds_since_epoch: u64) -> DateTime<Local> {
    let system_time = UNIX_EPOCH + Duration::from_secs(seconds_since_epoch);
    system_time.into()
}

pub(super) fn find_current_process_user(sys: &System) -> Result<Uid> {
    let current_process_pid =
        sysinfo::get_current_pid().map_err(|e| anyhow!("Unsupported platform! {}", e))?;
    sys.process(current_process_pid)
        .and_then(|cp| cp.user_id().cloned())
        .context("Current process not found!")
}

#[cfg(test)]
pub mod tests {

    use std::{ops::Mul, str::FromStr, time::Duration};

    use super::*;

    pub struct MockProcessInfo {
        pub pid: u32,
        pub parent_pid: Option<u32>,
        pub user_id: Uid,
        pub is_thread: bool,
        pub cmd: String,
        pub cmd_path: Option<String>,
        pub args: Vec<String>,
        pub memory: u64,
        pub start_time: u64,
        pub run_time: u64,
    }

    impl ProcessInfo for MockProcessInfo {
        fn is_thread(&self) -> bool {
            self.is_thread
        }

        fn user_id(&self) -> Option<&Uid> {
            Some(&self.user_id)
        }

        fn cmd(&self) -> &str {
            &self.cmd
        }

        fn cmd_path(&self) -> Option<&str> {
            self.cmd_path.as_deref()
        }

        fn pid(&self) -> u32 {
            self.pid
        }

        fn parent_id(&self) -> Option<u32> {
            self.parent_pid
        }

        fn memory(&self) -> u64 {
            self.memory
        }

        fn start_time(&self) -> u64 {
            self.start_time
        }

        fn run_time(&self) -> u64 {
            self.run_time
        }

        fn args(&self) -> Vec<&str> {
            self.args.iter().map(|a| a.as_str()).collect()
        }
    }

    impl Default for MockProcessInfo {
        fn default() -> MockProcessInfo {
            MockProcessInfo {
                pid: 1,
                parent_pid: None,
                user_id: Uid::from_str("1").unwrap(),
                is_thread: false,
                cmd: "xxx".to_string(),
                cmd_path: Some("xxx".to_string()),
                args: vec!["xxx".to_string(), "xxx2".to_string()],
                memory: 0,
                start_time: 0,
                run_time: 0,
            }
        }
    }

    impl MockProcessInfo {
        pub fn with_args(mut self, args: &[&str]) -> MockProcessInfo {
            self.args = args.iter().map(|s| s.to_string()).collect();
            self
        }
    }

    #[test]
    fn test_get_process_args() {
        let mut prc = MockProcessInfo {
            cmd: "exe".into(),
            cmd_path: Some("/path/to/cmd".to_string()),
            ..Default::default()
        };

        prc = prc.with_args(&["exe", "a1", "a2"]);
        assert_eq!(get_process_args(&prc), Some("a1 a2".to_string()));

        prc = prc.with_args(&["/path/to/cmd", "a1"]);
        assert_eq!(get_process_args(&prc), Some("a1".to_string()));

        prc = prc.with_args(&["--a1", "-a2"]);
        assert_eq!(get_process_args(&prc), Some("--a1 -a2".to_string()));

        prc = prc.with_args(&[]);
        assert_eq!(get_process_args(&prc), None);
        prc = prc.with_args(&[" "]);
        assert_eq!(get_process_args(&prc), None);
    }

    #[test]
    fn test_process_run_time() {
        let run_time = |hours: u64, minutes: u64, seconds: u64| {
            let duration = as_duration(hours, minutes, seconds);
            process_run_time(duration.as_secs(), UNIX_EPOCH + duration.mul(2))
        };
        assert_eq!(run_time(0, 0, 0), "0s");
        assert_eq!(run_time(0, 0, 5), "5s");
        assert_eq!(run_time(0, 30, 5), "30m 5s");
        assert_eq!(run_time(0, 59, 0), "59m 0s");
        assert_eq!(run_time(3, 0, 0), "3h 0m 0s");
        assert_eq!(run_time(3, 0, 30), "3h 0m 30s");
        assert_eq!(run_time(3, 30, 0), "3h 30m 0s");
        assert_eq!(run_time(3, 45, 15), "3h 45m 15s");
    }

    #[test]
    fn test_to_system_local_time() {
        let system_time_utc = |hours: u64, minutes: u64, seconds: u64| {
            let seconds_since_epoch = as_duration(hours, minutes, seconds).as_secs();
            to_system_local_time(seconds_since_epoch)
                //NOTE: without this it will fail at CI/CD pipeline where local time is different
                .to_utc()
                .format("%H:%M:%S")
                .to_string()
        };
        assert_eq!(system_time_utc(0, 0, 0), "00:00:00");
        assert_eq!(system_time_utc(1, 45, 15), "01:45:15");
        assert_eq!(system_time_utc(5, 29, 59), "05:29:59");
    }

    fn as_duration(hours: u64, minutes: u64, seconds: u64) -> Duration {
        Duration::from_secs(hours * 3600 + minutes * 60 + seconds)
    }
}
