use std::collections::{HashMap, HashSet};

use itertools::Itertools;
use listeners::Listener;

#[derive(Default)]
pub struct ProcessPorts {
    ports: HashMap<u32, String>,
}

impl ProcessPorts {
    pub fn new_refreshed() -> ProcessPorts {
        let listeners = listeners::get_all()
            //NOTE: we ignore errors coming from listeners
            .unwrap_or_default();
        Self {
            ports: create_sorted_process_ports(listeners),
        }
    }

    pub fn get(&self, pid: &u32) -> Option<&String> {
        self.ports.get(pid)
    }
}

//NOTE: we sort this so order of ports is deterministic and doesn't change during refresh
fn create_sorted_process_ports(ports: HashSet<Listener>) -> HashMap<u32, String> {
    ports
        .into_iter()
        .map(|l| (l.process.pid, l.socket.port()))
        .into_group_map()
        .into_iter()
        .map(|(pid, ports)| (pid, ports.into_iter().sorted_by(|a, b| a.cmp(b)).join(", ")))
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use listeners::{Listener, Process};

    use super::*;

    #[test]
    fn should_create_sorted_process_ports() {
        let value = [
            create_listener(1, 8080),
            create_listener(1, 100),
            create_listener(1, 50),
            create_listener(2, 1234),
        ];
        let process_ports = create_sorted_process_ports(HashSet::from(value));
        assert_eq!(process_ports.len(), 2);
        assert_eq!(process_ports.get(&1).unwrap(), "50, 100, 8080");
        assert_eq!(process_ports.get(&2).unwrap(), "1234");
    }

    fn create_listener(pid: u32, port: u16) -> Listener {
        Listener {
            process: Process {
                pid,
                name: format!("p1{pid}"),
                path: format!("p1{pid}"),
            },
            socket: format!("127.0.0.1:{port}").parse().unwrap(),
            protocol: listeners::Protocol::TCP,
        }
    }
}
