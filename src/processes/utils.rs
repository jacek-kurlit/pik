use std::time::{Duration, UNIX_EPOCH};

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Local};
use sysinfo::{System, Uid};

use super::ProcessInfo;

// NOTE: Some processes have path to binary as first argument, but also some processes has different name than cmd (for exmaple firefox)
pub(super) fn get_process_args(
    prc: &impl ProcessInfo,
    cmd_path: &Option<String>,
    cmd: &str,
) -> String {
    let args = prc.args();
    let cmd_path = cmd_path.as_deref().unwrap_or("");
    if args
        .first()
        .is_some_and(|arg1| arg1 == cmd_path || arg1.ends_with(cmd))
    {
        return args[1..].join(", ");
    }
    args.join(", ")
}

pub(super) fn format_seconds_as_hh_mm_ss(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let seconds = seconds % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

pub(super) fn format_as_epoch_time(time: u64) -> String {
    let system_time = UNIX_EPOCH + Duration::from_secs(time);
    let datetime: DateTime<Local> = system_time.into();
    datetime.format("%H:%M:%S").to_string()
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

    use std::str::FromStr;

    use super::*;

    pub struct MockProcessInfo {
        pid: u32,
        parent_pid: Option<u32>,
        user_id: Uid,
        is_thread: bool,
        cmd: String,
        cmd_path: Option<String>,
        args: Vec<String>,
        memory: u64,
        start_time: u64,
        run_time: u64,
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

        fn args(&self) -> &[String] {
            &self.args
        }
    }

    impl MockProcessInfo {
        pub fn new() -> MockProcessInfo {
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

        pub fn with_args(mut self, args: &[&str]) -> MockProcessInfo {
            self.args = args.iter().map(|s| s.to_string()).collect();
            self
        }
    }

    #[test]
    fn test_get_process_args() {
        let mut prc = MockProcessInfo::new();

        prc = prc.with_args(&["cmd", "a1", "a2"]);
        assert_eq!(get_process_args(&prc, &None, "cmd"), "a1, a2");
        prc = prc.with_args(&["/path/to/cmd", "a1"]);
        assert_eq!(
            get_process_args(&prc, &Some("/path/to/cmd".to_string()), "exe"),
            "a1"
        );
        prc = prc.with_args(&["--a1", "-a2"]);
        assert_eq!(get_process_args(&prc, &None, "cmd"), "--a1, -a2");
    }

    #[test]
    fn test_format_seconds_as_hh_mm_ss() {
        assert_eq!(format_seconds_as_hh_mm_ss(0), "00:00:00");
        assert_eq!(format_seconds_as_hh_mm_ss(3600), "01:00:00");
        assert_eq!(
            format_seconds_as_hh_mm_ss(3600 * 2 + 60 * 30 + 10),
            "02:30:10"
        );
    }

    #[test]
    fn test_fromat_as_epoch_time() {
        assert_eq!(format_as_epoch_time(0), "01:00:00");
        assert_eq!(format_as_epoch_time(3600), "02:00:00");
        assert_eq!(format_as_epoch_time(3600 * 2 + 60 * 30 + 10), "03:30:10");
    }
}
