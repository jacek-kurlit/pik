use std::collections::{HashMap, HashSet};
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, anyhow};
use chrono::{DateTime, Local};
use itertools::Itertools;
use listeners::{Listener, Process, Protocol};
use regex::Regex;
use sysinfo::{Pid, System, Uid};

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

#[derive(Debug, PartialEq)]
struct ContainerProcess {
    is_thread: bool,
    user_id: Option<Uid>,
    container_name: String,
    cmd_path: String,
    pid: u32,
    container_runtime_pid: Option<u32>,
    memory: u64,
    start_time: u64,
    run_time: u64,
    args: Vec<String>,
    container_id: String,
}

impl ContainerProcess {
    fn new(container_name: &str, cmd_path: &str, pid: u32, args: &Vec<String>) -> Self {
        ContainerProcess {
            is_thread: false,
            user_id: None,
            container_name: container_name.to_string(),
            cmd_path: cmd_path.to_string(),
            pid,
            container_runtime_pid: None,
            memory: 0,
            start_time: 0,
            run_time: 0,
            args: args.clone(),
            container_id: String::new(),
        }
    }
}

impl ProcessInfo for ContainerProcess {
    fn is_thread(&self) -> bool {
        self.is_thread
    }

    fn user_id(&self) -> Option<&Uid> {
        self.user_id.as_ref()
    }

    fn cmd(&self) -> &str {
        self.container_name.as_str()
    }

    fn cmd_path(&self) -> Option<&str> {
        Some(&self.cmd_path)
    }

    fn pid(&self) -> u32 {
        self.pid
    }

    fn parent_id(&self) -> Option<u32> {
        self.container_runtime_pid
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
        self.args.iter().map(|arg| arg.as_str()).collect()
    }

    fn container_id(&self) -> Option<&str> {
        Some(&self.container_id)
    }
}

pub(super) fn get_container_processes(
    processes: &HashMap<Pid, impl ProcessInfo>,
    user_id: &Option<Uid>,
) -> HashMap<Pid, impl ProcessInfo> {
    let mut container_ids = get_container_ids();
    let mut container_processes = HashMap::with_capacity(container_ids.len());

    if container_ids.len() == 0 {
        return container_processes;
    }

    for line in get_process_information_of_containers(&container_ids) {
        let Some(mut container) = extract_process_information_from_line(&line) else {
            continue;
        };

        let pid = Pid::from_u32(container.pid);
        let Some(process_for_container) = processes.get(&pid) else {
            continue;
        };

        container.is_thread = process_for_container.is_thread();
        container.user_id = user_id.clone();
        container.container_runtime_pid = process_for_container.parent_id();
        container.memory = process_for_container.memory();
        container.start_time = process_for_container.start_time();
        container.run_time = process_for_container.run_time();
        container.container_id = container_ids.remove(0);

        container_processes.insert(pid, container);
    }

    container_processes
}

pub(super) fn get_container_ports() -> HashSet<Listener> {
    let container_ids = get_container_ids();
    let mut container_ports = HashSet::with_capacity(container_ids.len());

    if container_ids.len() == 0 {
        return container_ports;
    }

    for line in get_network_information_of_containers(&container_ids) {
        let Some(listeners) = extract_network_information_from_line(&line) else {
            continue;
        };

        container_ports.extend(listeners);
    }

    container_ports
}

fn get_container_ids() -> Vec<String> {
    let container_ids = Command::new("docker")
        .arg("ps")
        .arg("--no-trunc")
        .arg("-q")
        .stderr(Stdio::null())
        .output()
        .map_or(String::new(), |output| {
            String::from_utf8(output.stdout).map_or(String::new(), |val| val)
        });

    container_ids
        .split('\n')
        .map(|id| id.to_string())
        .filter(|id| !id.is_empty())
        .collect::<Vec<String>>()
}

fn get_process_information_of_containers(container_ids: &Vec<String>) -> Vec<String> {
    let container_information = Command::new("docker")
        .arg("inspect")
        .arg("-f \"cmd: '{{.Name}}';cmd_path: '{{.Path}}';pid: '{{.State.Pid}}';args: '{{join .Args \",\"}}'\"")
        .args(container_ids)
        .stderr(Stdio::null())
        .output()
        .map_or(String::new(), |output| String::from_utf8(output.stdout)
            .map_or(String::new(), |val| val));

    container_information
        .split('\n')
        .map(|information| {
            information
                .trim_start_matches(" \"")
                .trim_end_matches('"')
                .to_string()
        })
        .filter(|information| !information.is_empty())
        .collect::<Vec<String>>()
}

fn get_network_information_of_containers(container_ids: &Vec<String>) -> Vec<String> {
    let container_information = Command::new("docker")
        .arg("inspect")
        .arg("-f 'pid: \'{{.State.Pid}}\';ports: \'{{range $p, $conf := .NetworkSettings.Ports}}\'{{$p}}\'->\'{{(index $conf 0).HostIp}}:{{(index $conf 0).HostPort}}\'{{end}}\';'")
        .args(container_ids)
        .stderr(Stdio::null())
        .output()
        .map_or(String::new(), |output| String::from_utf8(output.stdout)
            .map_or(String::new(), |val| val));

    container_information
        .split('\n')
        .map(|information| {
            information
                .trim_start_matches(" '")
                .trim_end_matches('\'')
                .to_string()
        })
        .filter(|information| !information.is_empty())
        .collect::<Vec<String>>()
}

fn extract_process_information_from_line(line: &str) -> Option<ContainerProcess> {
    let regex_to_extract_information_from_line = Regex::new(r"(?U)^cmd: '/(?<cmd>.*)';cmd_path: '(?<cmd_path>.*)';pid: '(?<pid>\d+)';args: '(?<args>.*)'$")
        .unwrap();

    let Some(line_information) = regex_to_extract_information_from_line.captures(line) else {
        return None;
    };

    let cmd = line_information.name("cmd").unwrap().as_str();
    let cmd_path = line_information.name("cmd_path").unwrap().as_str();
    let pid = line_information
        .name("pid")
        .map(|pid| pid.as_str().parse::<u32>().unwrap())
        .unwrap();
    let args = line_information
        .name("args")
        .unwrap()
        .as_str()
        .split(',')
        .map(|id| id.to_string())
        .filter(|id| !id.is_empty())
        .collect::<Vec<String>>();

    Some(ContainerProcess::new(cmd, cmd_path, pid, &args))
}

fn extract_network_information_from_line(line: &str) -> Option<Vec<Listener>> {
    let regex_to_extract_information_from_line =
        Regex::new(r"(?U)^pid: '(?<pid>\d+)';ports: '(?<ports>.*)';$").unwrap();

    let Some(line_information) = regex_to_extract_information_from_line.captures(line) else {
        return None;
    };

    let pid = line_information
        .name("pid")
        .map(|pid| pid.as_str().parse::<u32>().unwrap())
        .unwrap();
    let ports = line_information
        .name("ports")
        .map(|ports| ports.as_str())
        .unwrap();

    let regex_to_extract_port_information =
        Regex::new(r"(?U)'(:?\d+)/(?<protocol>(:?tcp)|(:?udp))'->'(?<addr>.*)'").unwrap();
    if !regex_to_extract_port_information.is_match(ports) {
        return None;
    }

    let mut listeners = Vec::new();
    for capture in regex_to_extract_port_information.captures_iter(ports) {
        let protocol = match capture
            .name("protocol")
            .map(|protocol| protocol.as_str())
            .unwrap()
        {
            "tcp" => Protocol::TCP,
            "udp" => Protocol::UDP,
            protocol => panic!(
                "Error while mapping {} to protocol. This should not be possible",
                protocol
            ),
        };
        let addr = capture.name("addr").map(|addr| addr.as_str()).unwrap();

        listeners.push(create_listener(pid, addr, protocol));
    }

    Some(listeners)
}

fn create_listener(pid: u32, socket_addr: &str, protocol: Protocol) -> Listener {
    Listener {
        process: Process {
            pid,
            name: String::new(),
            path: String::new(),
        },
        socket: socket_addr.parse().unwrap(),
        protocol,
    }
}

#[cfg(test)]
pub mod tests {

    use std::{ops::Mul, str::FromStr, time::Duration};

    use super::*;

    /// Creates a `Uid` from a number, using a platform-appropriate format.
    /// On Windows, UIDs are SID strings (e.g. `S-1-5-1000`); on Unix they are numeric.
    pub fn make_uid(n: u32) -> Uid {
        #[cfg(windows)]
        return Uid::from_str(&format!("S-1-5-{n}")).unwrap();
        #[cfg(not(windows))]
        return Uid::from_str(&n.to_string()).unwrap();
    }

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
                user_id: make_uid(1),
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

    #[test]
    fn test_extract_process_information_from_line() {
        let line =
            "cmd: '/TestContainer';cmd_path: 'docker-entrypoint.sh';pid: '5862';args: 'postgres'";
        let expected_container_process = ContainerProcess::new(
            "TestContainer",
            "docker-entrypoint.sh",
            5862,
            &vec![String::from("postgres")],
        );

        let container_process = extract_process_information_from_line(line);

        assert_eq!(container_process, Some(expected_container_process));
    }

    #[test]
    fn test_extract_network_information_from_line() {
        let line = "pid: '12801';ports: ''80/tcp'->'0.0.0.0:8080''80/udp'->'0.0.0.0:8080'';";
        let expected_listeners = vec![
            create_listener(12801, "0.0.0.0:8080", Protocol::TCP),
            create_listener(12801, "0.0.0.0:8080", Protocol::UDP),
        ];

        let listeners = extract_network_information_from_line(line);

        assert_eq!(listeners, Some(expected_listeners));
    }

    fn as_duration(hours: u64, minutes: u64, seconds: u64) -> Duration {
        Duration::from_secs(hours * 3600 + minutes * 60 + seconds)
    }
}
