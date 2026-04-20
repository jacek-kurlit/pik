use super::utils::create_listener;

use listeners::{Listener, Protocol};
use regex::Regex;

use std::collections::{HashMap, HashSet};
use std::process::{Command, Stdio};

pub(super) fn get_container_pids() -> HashMap<u32, String> {
    let mut container_ids = get_container_ids();
    if container_ids.is_empty() {
        return HashMap::new();
    }

    let mut container_pids = HashMap::with_capacity(container_ids.len());
    let pids = get_process_information_of_containers(&container_ids);

    for pid in pids {
        let pid = pid.parse::<u32>().unwrap();
        container_pids.insert(pid, container_ids.remove(0));
    }

    container_pids
}

pub(super) fn kill_container(container_id: &str) -> bool {
    let output = Command::new("docker")
        .arg("kill")
        .arg(container_id)
        .output()
        .map_or(String::new(), |output| {
            String::from_utf8(output.stdout).map_or(String::new(), |val| val)
        });

    !output.is_empty()
}

pub(super) fn get_container_ports() -> HashSet<Listener> {
    let container_ids = get_container_ids();
    let mut container_ports = HashSet::with_capacity(container_ids.len());

    if container_ids.is_empty() {
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
        .args(["-f", "{{.State.Pid}}"])
        .args(container_ids)
        .stderr(Stdio::null())
        .output()
        .map_or(String::new(), |output| {
            String::from_utf8(output.stdout).map_or(String::new(), |val| val)
        });

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
        .args(["-f", "pid: \'{{.State.Pid}}\';ports: \'{{range $p, $conf := .NetworkSettings.Ports}}\'{{$p}}\'->\'{{(index $conf 0).HostIp}}:{{(index $conf 0).HostPort}}\'{{end}}\';"])
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

fn extract_network_information_from_line(line: &str) -> Option<Vec<Listener>> {
    let regex_to_extract_information_from_line =
        Regex::new(r"(?U)^pid: '(?<pid>\d+)';ports: '(?<ports>.*)';$").unwrap();

    let line_information = regex_to_extract_information_from_line.captures(line)?;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
