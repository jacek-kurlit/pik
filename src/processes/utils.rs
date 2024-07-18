use std::time::{Duration, UNIX_EPOCH};

use chrono::{DateTime, Local};

// NOTE: Some processes have path to binary as first argument, but also some processes has different name than cmd (for exmaple firefox)
pub(super) fn get_process_args(
    prc: &sysinfo::Process,
    cmd_path: &Option<String>,
    cmd: &str,
) -> String {
    let args = prc.cmd();
    let cmd_path = cmd_path.as_deref().unwrap_or("");
    if args
        .first()
        .is_some_and(|arg1| arg1 == cmd_path || arg1.ends_with(cmd))
    {
        return args[1..].join(", ");
    }
    args.join(", ")
}

pub(super) fn format_as_hh_mm_ss(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let seconds = seconds % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

pub(super) fn format_as_epoch_time(time: u64) -> String {
    let system_time = UNIX_EPOCH + Duration::from_secs(time);
    let datetime_utc: DateTime<Local> = system_time.into();
    datetime_utc.format("%H:%M:%S").to_string()
}
