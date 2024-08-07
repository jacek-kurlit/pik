use sysinfo::Uid;

use super::ProcessInfo;

pub(super) struct QueryFilter {
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

impl QueryFilter {
    pub fn new(query: &str) -> Self {
        let (search_by, query) = match query.chars().next() {
            Some(':') => (SearchBy::Port, &query[1..]),
            Some('/') => (SearchBy::Path, &query[1..]),
            Some('-') => (SearchBy::Args, &query[1..]),
            Some(_) => (SearchBy::Cmd, query),
            None => (SearchBy::None, query),
        };
        Self {
            query: query.to_lowercase(),
            search_by,
        }
    }

    pub(super) fn accept(&self, prc: &impl ProcessInfo, ports: Option<&str>) -> bool {
        match self.search_by {
            SearchBy::Cmd => prc.cmd().to_lowercase().contains(&self.query),
            SearchBy::Path => prc
                .cmd_path()
                .unwrap_or("")
                .to_lowercase()
                .contains(&self.query),
            SearchBy::Args => prc
                .args()
                .iter()
                .map(|a| a.to_lowercase())
                .any(|a| a.contains(&self.query)),
            SearchBy::Port => ports
                .as_ref()
                .map(|p| p.contains(&self.query))
                .unwrap_or(false),
            SearchBy::None => true,
        }
    }
}

#[derive(Copy, Clone, Default)]
pub struct FilterOptions {
    //NOTE: On linux threads can be listed as processes and thus needs filtering
    pub ignore_threads: bool,
    pub user_processes_only: bool,
}

pub(super) struct OptionsFilter<'a> {
    opt: FilterOptions,
    current_user_id: &'a Uid,
}

impl<'a> OptionsFilter<'a> {
    pub fn new(opt: FilterOptions, current_user_id: &'a Uid) -> Self {
        Self {
            opt,
            current_user_id,
        }
    }

    pub fn accept(&self, prc: &impl ProcessInfo) -> bool {
        {
            if self.opt.ignore_threads && prc.is_thread() {
                return false;
            }
            if !self.opt.user_processes_only {
                return true;
            }
            prc.user_id() == Some(self.current_user_id)
        }
    }
}

#[cfg(test)]
pub mod tests {

    use std::str::FromStr;

    use crate::processes::utils::tests::MockProcessInfo;

    use super::*;

    #[test]
    fn should_create_proper_query_filter() {
        let filter = QueryFilter::new("FOO");
        assert_eq!(filter.search_by, SearchBy::Cmd);
        assert_eq!(filter.query, "foo");

        let filter = QueryFilter::new("/Foo");
        assert_eq!(filter.search_by, SearchBy::Path);
        assert_eq!(filter.query, "foo");

        let filter = QueryFilter::new("-fOo");
        assert_eq!(filter.search_by, SearchBy::Args);
        assert_eq!(filter.query, "foo");

        let filter = QueryFilter::new(":foo");
        assert_eq!(filter.search_by, SearchBy::Port);
        assert_eq!(filter.query, "foo");

        let filter = QueryFilter::new("");
        assert_eq!(filter.search_by, SearchBy::None);
        assert_eq!(filter.query, "");
    }

    #[test]
    fn query_filter_search_by_cmd() {
        let filter = QueryFilter::new("test");
        let mut process = MockProcessInfo {
            cmd: "TeSt".to_string(),
            ..Default::default()
        };
        assert!(filter.accept(&process, None));

        process.cmd = "test".to_string();
        assert!(filter.accept(&process, None));

        process.cmd = "TEST".to_string();
        assert!(filter.accept(&process, None));

        process.cmd = "Testificator".to_string();
        assert!(filter.accept(&process, None));

        process.cmd = "online_TESTER".to_string();
        assert!(filter.accept(&process, None));

        process.cmd = "xxx".to_string();
        assert!(!filter.accept(&process, None));
    }

    #[test]
    fn query_filter_search_by_path() {
        let filter = QueryFilter::new("/test");
        let mut process = MockProcessInfo {
            cmd_path: Some("/TeSt".to_string()),
            ..Default::default()
        };
        assert!(filter.accept(&process, None));

        process.cmd_path = Some("/test".to_string());
        assert!(filter.accept(&process, None));

        process.cmd_path = Some("/TEST".to_string());
        assert!(filter.accept(&process, None));

        process.cmd_path = Some("/testing_dir".to_string());
        assert!(filter.accept(&process, None));

        process.cmd_path = Some("/cargo/tests".to_string());
        assert!(filter.accept(&process, None));

        process.cmd_path = Some("/xxx".to_string());
        assert!(!filter.accept(&process, None));
    }

    #[test]
    fn query_filter_search_by_args() {
        let filter = QueryFilter::new("-test");
        let mut process = MockProcessInfo::default();

        process = process.with_args(&["-TeSt"]);
        assert!(filter.accept(&process, None));

        process = process.with_args(&["-test"]);
        assert!(filter.accept(&process, None));

        process = process.with_args(&["-TEST"]);
        assert!(filter.accept(&process, None));

        process = process.with_args(&["arg1, arg2, --testifier"]);
        assert!(filter.accept(&process, None));

        process = process.with_args(&["testimony"]);
        assert!(filter.accept(&process, None));

        process = process.with_args(&["-xxx"]);
        assert!(!filter.accept(&process, None));
    }

    #[test]
    fn query_filter_search_by_port() {
        let filter = QueryFilter::new(":12");
        let process = MockProcessInfo::default();

        assert!(filter.accept(&process, Some("1234")));

        assert!(filter.accept(&process, Some("3312")));

        assert!(filter.accept(&process, Some("5125")));

        assert!(filter.accept(&process, Some("1111, 2222, 1234")));

        assert!(!filter.accept(&process, Some("7777")));
    }

    #[test]
    fn query_filter_search_by_none() {
        let filter = QueryFilter::new("");
        let mut process = MockProcessInfo::default();
        assert!(filter.accept(&process, None));

        process.cmd = "TeSt".to_string();
        assert!(filter.accept(&process, None));

        process.cmd_path = Some("/TeSt".to_string());
        assert!(filter.accept(&process, None));

        process = process.with_args(&["-TeSt"]);
        assert!(filter.accept(&process, None));

        assert!(filter.accept(&process, Some("1234")));
    }

    #[test]
    fn options_filter_should_ignore_thread_processes() {
        let current_user_id = Uid::from_str("1").unwrap();
        let filter = OptionsFilter::new(
            FilterOptions {
                ignore_threads: true,
                ..Default::default()
            },
            &current_user_id,
        );
        let prc = MockProcessInfo {
            is_thread: true,
            ..Default::default()
        };

        assert!(!filter.accept(&prc));
    }

    #[test]
    fn options_filter_should_accept_all_threads_processes() {
        let current_user_id = Uid::from_str("1").unwrap();
        let filter = OptionsFilter::new(
            FilterOptions {
                ignore_threads: false,
                ..Default::default()
            },
            &current_user_id,
        );
        let prc = MockProcessInfo {
            is_thread: true,
            ..Default::default()
        };

        assert!(filter.accept(&prc));
    }

    #[test]
    fn options_filter_should_accept_only_current_user_processes() {
        let current_user_id = Uid::from_str("1000").unwrap();
        let filter = OptionsFilter::new(
            FilterOptions {
                user_processes_only: true,
                ..Default::default()
            },
            &current_user_id,
        );
        let mut prc = MockProcessInfo {
            user_id: current_user_id.clone(),
            ..Default::default()
        };
        assert!(filter.accept(&prc));

        prc.user_id = Uid::from_str("1001").unwrap();
        assert!(!filter.accept(&prc));
    }

    #[test]
    fn options_filter_should_accept_all_processes() {
        let current_user_id = Uid::from_str("1000").unwrap();
        let filter = OptionsFilter::new(
            FilterOptions {
                user_processes_only: false,
                ..Default::default()
            },
            &current_user_id,
        );
        let mut prc = MockProcessInfo {
            user_id: current_user_id.clone(),
            ..Default::default()
        };
        assert!(filter.accept(&prc));

        prc.user_id = Uid::from_str("1001").unwrap();
        assert!(filter.accept(&prc));
    }
}
