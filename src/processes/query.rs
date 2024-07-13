use super::Process;

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

    pub(super) fn apply(&self, prc: &Process) -> bool {
        match self.filter_by {
            FilterBy::Cmd => prc.cmd.to_lowercase().contains(&self.query),
            FilterBy::Path => prc.exe_path.to_lowercase().contains(&self.query),
            FilterBy::Args => prc.args.to_lowercase().contains(&self.query),
            FilterBy::Port => prc.ports.contains(&self.query),
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

    #[test]
    fn test_apply_filter_by_cmd() {
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
    fn test_apply_filter_by_path() {
        let filter = ProcessFilter::new("/test");
        let mut process = some_process();

        process.exe_path = "/TeSt".to_string();
        assert!(filter.apply(&process));

        process.exe_path = "/test".to_string();
        assert!(filter.apply(&process));

        process.exe_path = "/TEST".to_string();
        assert!(filter.apply(&process));

        process.exe_path = "/testing_dir".to_string();
        assert!(filter.apply(&process));

        process.exe_path = "/cargo/tests".to_string();
        assert!(filter.apply(&process));

        process.exe_path = "/xxx".to_string();
        assert!(!filter.apply(&process));
    }

    #[test]
    fn test_apply_filter_by_args() {
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
    fn test_apply_filter_by_port() {
        let filter = ProcessFilter::new(":12");
        let mut process = some_process();

        process.ports = "1234".to_string();
        assert!(filter.apply(&process));

        process.ports = "3312".to_string();
        assert!(filter.apply(&process));

        process.ports = "5125".to_string();
        assert!(filter.apply(&process));

        process.ports = "1111, 2222, 1234".to_string();
        assert!(filter.apply(&process));

        process.ports = "7777".to_string();
        assert!(!filter.apply(&process));
    }

    #[test]
    fn test_apply_filter_by_none() {
        let filter = ProcessFilter::new("");
        let mut process = some_process();
        assert!(filter.apply(&process));

        process.cmd = "TeSt".to_string();
        assert!(filter.apply(&process));

        process.exe_path = "/TeSt".to_string();
        assert!(filter.apply(&process));

        process.args = "-TeSt".to_string();
        assert!(filter.apply(&process));

        process.ports = "1234".to_string();
        assert!(filter.apply(&process));
    }

    fn some_process() -> Process {
        Process {
            pid: 1,
            user_name: "xxx".to_string(),
            cmd: "xxx".to_string(),
            exe_path: "xxx".to_string(),
            args: "xxx, xxx2, --xxx3".to_string(),
            ports: "0000".to_string(),
        }
    }
}
