use super::{ProcessInfo, ProcessPorts};

pub(super) struct ProcessFilter {
    query: String,
    filter_by: FilterBy,
}

#[derive(PartialEq, Eq, Debug)]
enum FilterBy {
    Cmd,
    Port,
    Path,
    Args,
    None,
}

impl ProcessFilter {
    pub fn new(query: &str) -> Self {
        let (filterby, query) = match query.chars().next() {
            Some(':') => (FilterBy::Port, &query[1..]),
            Some('/') => (FilterBy::Path, query),
            Some('-') => (FilterBy::Args, &query[1..]),
            Some(_) => (FilterBy::Cmd, query),
            None => (FilterBy::None, query),
        };
        Self {
            query: query.to_lowercase(),
            filter_by: filterby,
        }
    }

    //TODO: maybe instead of filtring bool we can filterMap? with Some(Process) or None
    pub(super) fn apply(&self, prc: &impl ProcessInfo, process_ports: &ProcessPorts) -> bool {
        //NOTE: On linux threads can be listed as processes and thus needs filtering
        if prc.is_thread() {
            return false;
        }
        match self.filter_by {
            FilterBy::Cmd => prc.cmd().to_lowercase().contains(&self.query),
            FilterBy::Path => prc
                .exe_path()
                .map(|e| e.contains(&self.query))
                .unwrap_or(false),
            FilterBy::Args => prc
                .args()
                .iter()
                .any(|arg| arg.to_lowercase().contains(&self.query)),
            FilterBy::Port => process_ports
                .get(&prc.pid_u32())
                .map(|ports| ports.iter().any(|port| port.contains(&self.query)))
                .unwrap_or(false),
            FilterBy::None => true,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn should_create_proper_filter() {
        let filter = ProcessFilter::new("FOO");
        assert_eq!(filter.filter_by, FilterBy::Cmd);
        assert_eq!(filter.query, "foo");

        let filter = ProcessFilter::new("/Foo");
        assert_eq!(filter.filter_by, FilterBy::Path);
        assert_eq!(filter.query, "/foo");

        let filter = ProcessFilter::new("-fOo");
        assert_eq!(filter.filter_by, FilterBy::Args);
        assert_eq!(filter.query, "foo");

        let filter = ProcessFilter::new(":foo");
        assert_eq!(filter.filter_by, FilterBy::Port);
        assert_eq!(filter.query, "foo");

        let filter = ProcessFilter::new("");
        assert_eq!(filter.filter_by, FilterBy::None);
        assert_eq!(filter.query, "");
    }
    //
    // //FIXME: this is pretty cool but how about mockall?
    // struct MockProcessInfo {
    //     cmd: String,
    //     exe_path: Option<String>,
    //     args: Vec<String>,
    //     is_thread: bool,
    // }
    //
    // impl ProcessInfo for MockProcessInfo {
    //     fn cmd(&self) -> &str {
    //         &self.cmd
    //     }
    //
    //     fn exe_path(&self) -> Option<&str> {
    //         self.exe_path.as_deref()
    //     }
    //
    //     fn args(&self) -> Vec<&str> {
    //         self.args.iter().map(|s| s.as_str()).collect()
    //     }
    //
    //     fn is_thread(&self) -> bool {
    //         self.is_thread
    //     }
    // }
    //
    // #[test]
    // fn test_apply_filter_by_cmd() {
    //     let filter = Filter {
    //         filter_by: FilterBy::Cmd,
    //         query: "test".to_lowercase(),
    //     };
    //     let process = MockProcessInfo {
    //         cmd: "test command".to_string(),
    //         exe_path: None,
    //         args: vec![],
    //         is_thread: false,
    //     };
    //
    //     assert!(filter.apply(&process));
    // }
    //
    // #[test]
    // fn test_apply_filter_by_path() {
    //     let filter = Filter {
    //         filter_by: FilterBy::Path,
    //         query: "test".to_lowercase(),
    //     };
    //     let process = MockProcessInfo {
    //         cmd: "command".to_string(),
    //         exe_path: Some("/path/to/test".to_string()),
    //         args: vec![],
    //         is_thread: false,
    //     };
    //
    //     assert!(filter.apply(&process));
    // }
    //
    // #[test]
    // fn test_apply_filter_by_args() {
    //     let filter = Filter {
    //         filter_by: FilterBy::Args,
    //         query: "test".to_lowercase(),
    //     };
    //     let process = MockProcessInfo {
    //         cmd: "command".to_string(),
    //         exe_path: None,
    //         args: vec!["arg1".to_string(), "testarg".to_string()],
    //         is_thread: false,
    //     };
    //
    //     assert!(filter.apply(&process));
    // }
    //
    // #[test]
    // fn test_apply_filter_by_port() {
    //     let filter = Filter {
    //         filter_by: FilterBy::Port,
    //         query: "test".to_lowercase(),
    //     };
    //     let process = MockProcessInfo {
    //         cmd: "command".to_string(),
    //         exe_path: None,
    //         args: vec![],
    //         is_thread: false,
    //     };
    //
    //     assert!(filter.apply(&process)); // This test should panic with "Not implemented yet"
    // }
    //
    // #[test]
    // fn test_apply_filter_by_none() {
    //     let filter = Filter {
    //         filter_by: FilterBy::None,
    //         query: "test".to_lowercase(),
    //     };
    //     let process = MockProcessInfo {
    //         cmd: "command".to_string(),
    //         exe_path: None,
    //         args: vec![],
    //         is_thread: false,
    //     };
    //
    //     assert!(filter.apply(&process));
    // }
}
