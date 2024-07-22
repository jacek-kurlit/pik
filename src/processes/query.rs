use super::Process;

pub(super) struct ProcessFilter {
    query: String,
    pub(super) search_by: SearchBy,
}

#[derive(PartialEq, Eq, Debug)]
pub enum SearchBy {
    Cmd,
    Port,
    Path,
    Args,
    None,
}

impl ProcessFilter {
    pub fn new(query: &str) -> Self {
        let (search_by, query) = match query.chars().next() {
            Some(':') => (SearchBy::Port, &query[1..]),
            Some('/') => (SearchBy::Path, query),
            Some('-') => (SearchBy::Args, &query[1..]),
            Some(_) => (SearchBy::Cmd, query),
            None => (SearchBy::None, query),
        };
        Self {
            query: query.to_lowercase(),
            search_by,
        }
    }

    pub(super) fn apply(&self, prc: &Process) -> bool {
        match self.search_by {
            SearchBy::Cmd => prc.cmd.to_lowercase().contains(&self.query),
            SearchBy::Path => prc
                .cmd_path
                .as_deref()
                .unwrap_or("")
                .to_lowercase()
                .contains(&self.query),
            SearchBy::Args => prc.args.to_lowercase().contains(&self.query),
            SearchBy::Port => prc
                .ports
                .as_ref()
                .map(|p| p.contains(&self.query))
                .unwrap_or(false),
            SearchBy::None => true,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn should_create_proper_filter() {
        let filter = ProcessFilter::new("FOO");
        assert_eq!(filter.search_by, SearchBy::Cmd);
        assert_eq!(filter.query, "foo");

        let filter = ProcessFilter::new("/Foo");
        assert_eq!(filter.search_by, SearchBy::Path);
        assert_eq!(filter.query, "/foo");

        let filter = ProcessFilter::new("-fOo");
        assert_eq!(filter.search_by, SearchBy::Args);
        assert_eq!(filter.query, "foo");

        let filter = ProcessFilter::new(":foo");
        assert_eq!(filter.search_by, SearchBy::Port);
        assert_eq!(filter.query, "foo");

        let filter = ProcessFilter::new("");
        assert_eq!(filter.search_by, SearchBy::None);
        assert_eq!(filter.query, "");
    }

    #[test]
    fn test_apply_search_by_cmd() {
        let filter = ProcessFilter::new("test");
        let mut process = some_process();

        process.cmd = "TeSt".to_string();
        assert!(filter.apply(&process));

        process.cmd = "test".to_string();
        assert!(filter.apply(&process));

        process.cmd = "TEST".to_string();
        assert!(filter.apply(&process));

        process.cmd = "Testificator".to_string();
        assert!(filter.apply(&process));

        process.cmd = "online_TESTER".to_string();
        assert!(filter.apply(&process));

        process.cmd = "xxx".to_string();
        assert!(!filter.apply(&process));
    }

    #[test]
    fn test_apply_search_by_path() {
        let filter = ProcessFilter::new("/test");
        let mut process = some_process();

        process.cmd_path = Some("/TeSt".to_string());
        assert!(filter.apply(&process));

        process.cmd_path = Some("/test".to_string());
        assert!(filter.apply(&process));

        process.cmd_path = Some("/TEST".to_string());
        assert!(filter.apply(&process));

        process.cmd_path = Some("/testing_dir".to_string());
        assert!(filter.apply(&process));

        process.cmd_path = Some("/cargo/tests".to_string());
        assert!(filter.apply(&process));

        process.cmd_path = Some("/xxx".to_string());
        assert!(!filter.apply(&process));
    }

    #[test]
    fn test_apply_search_by_args() {
        let filter = ProcessFilter::new("-test");
        let mut process = some_process();

        process.args = "-TeSt".to_string();
        assert!(filter.apply(&process));

        process.args = "-test".to_string();
        assert!(filter.apply(&process));

        process.args = "-TEST".to_string();
        assert!(filter.apply(&process));

        process.args = "arg1, arg2, --testifier".to_string();
        assert!(filter.apply(&process));

        process.args = "testimony".to_string();
        assert!(filter.apply(&process));

        process.args = "-xxx".to_string();
        assert!(!filter.apply(&process));
    }

    #[test]
    fn test_apply_search_by_port() {
        let filter = ProcessFilter::new(":12");
        let mut process = some_process();

        process.ports = Some("1234".to_string());
        assert!(filter.apply(&process));

        process.ports = Some("3312".to_string());
        assert!(filter.apply(&process));

        process.ports = Some("5125".to_string());
        assert!(filter.apply(&process));

        process.ports = Some("1111, 2222, 1234".to_string());
        assert!(filter.apply(&process));

        process.ports = Some("7777".to_string());
        assert!(!filter.apply(&process));
    }

    #[test]
    fn test_apply_search_by_none() {
        let filter = ProcessFilter::new("");
        let mut process = some_process();
        assert!(filter.apply(&process));

        process.cmd = "TeSt".to_string();
        assert!(filter.apply(&process));

        process.cmd_path = Some("/TeSt".to_string());
        assert!(filter.apply(&process));

        process.args = "-TeSt".to_string();
        assert!(filter.apply(&process));

        process.ports = Some("1234".to_string());
        assert!(filter.apply(&process));
    }

    fn some_process() -> Process {
        Process {
            pid: 1,
            parent_pid: None,
            user_name: "xxx".to_string(),
            cmd: "xxx".to_string(),
            cmd_path: Some("xxx".to_string()),
            args: "xxx, xxx2, --xxx3".to_string(),
            ports: Some("0000".to_string()),
            memory: 0,
            start_time: "00:00:00".to_string(),
            run_time: "00:00:00".to_string(),
        }
    }
}
