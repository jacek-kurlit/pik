use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use regex::Regex;
use sysinfo::Uid;

use super::{utils::get_process_args, MatchData, MatchType, MatchedBy, ProcessInfo};

pub(super) struct QueryFilter {
    query: String,
    pub(super) search_by: SearchBy,
    matcher: SkimMatcherV2,
}

#[derive(PartialEq, Eq, Debug)]
pub enum SearchBy {
    Cmd,
    Port,
    Path,
    Args,
    Everywhere,
    Pid,
    ProcessFamily,
    None,
}

impl QueryFilter {
    pub fn new(query: &str) -> Self {
        let (search_by, query) = match query.chars().next() {
            Some(':') => (SearchBy::Port, &query[1..]),
            Some('/') => (SearchBy::Path, &query[1..]),
            Some('-') => (SearchBy::Args, &query[1..]),
            Some('~') => (SearchBy::Everywhere, &query[1..]),
            Some('!') => (SearchBy::Pid, &query[1..]),
            Some('@') => (SearchBy::ProcessFamily, &query[1..]),
            Some(_) => (SearchBy::Cmd, query),
            None => (SearchBy::None, query),
        };
        let matcher = SkimMatcherV2::default();
        Self {
            query: query.to_lowercase(),
            search_by,
            matcher,
        }
    }
    pub(super) fn accept(&self, prc: &impl ProcessInfo, ports: Option<&str>) -> Option<MatchData> {
        match self.search_by {
            SearchBy::Cmd => self.fuzzy_match(prc.cmd(), MatchedBy::Cmd),
            SearchBy::Path => self.fuzzy_match_opt(prc.cmd_path(), MatchedBy::Path),
            SearchBy::Args => {
                self.fuzzy_match_opt(get_process_args(prc).as_deref(), MatchedBy::Args)
            }
            SearchBy::Port => self.fuzzy_match_opt(ports, MatchedBy::Port),
            SearchBy::Pid => self.exact_match_u32(prc.pid(), MatchedBy::Pid),
            SearchBy::ProcessFamily => self.exact_match_process_family(prc),
            SearchBy::Everywhere => self
                .fuzzy_match(prc.cmd(), MatchedBy::Cmd)
                .or_else(|| self.fuzzy_match_opt(prc.cmd_path(), MatchedBy::Path))
                .or_else(|| self.fuzzy_match_opt(ports, MatchedBy::Port))
                .or_else(|| {
                    self.fuzzy_match_opt(get_process_args(prc).as_deref(), MatchedBy::Args)
                }),
            SearchBy::None => Some(MatchData::new(
                MatchedBy::ProcessExistence,
                MatchType::Exists,
            )),
        }
    }

    fn fuzzy_match(&self, s: &str, matched_by: MatchedBy) -> Option<MatchData> {
        if self.query.is_empty() {
            return Some(MatchData::new(matched_by, MatchType::Exists));
        }
        self.matcher
            .fuzzy_indices(s, self.query.as_str())
            .map(|(score, positions)| {
                MatchData::new(matched_by, MatchType::Fuzzy { score, positions })
            })
    }

    fn fuzzy_match_opt(&self, s: Option<&str>, matched_by: MatchedBy) -> Option<MatchData> {
        s.and_then(|s| self.fuzzy_match(s, matched_by))
    }

    fn exact_match_u32(&self, s: u32, matched_by: MatchedBy) -> Option<MatchData> {
        if s.to_string() == self.query {
            Some(MatchData::new(matched_by, MatchType::Exact))
        } else {
            None
        }
    }

    fn exact_match_process_family(&self, prc: &impl ProcessInfo) -> Option<MatchData> {
        if prc.pid().to_string() == self.query {
            return Some(MatchData::new(MatchedBy::Pid, MatchType::Exact));
        }
        if prc
            .parent_id()
            .map(|pid| pid.to_string() == self.query)
            .unwrap_or(false)
        {
            return Some(MatchData::new(MatchedBy::ParentPid, MatchType::Exact));
        }
        None
    }
}

#[derive(Debug)]
pub struct IgnoreOptions {
    //NOTE: On linux threads can be listed as processes and thus needs filtering
    pub ignore_threads: bool,
    pub ignore_other_users: bool,
    pub paths: Vec<Regex>,
}

impl Default for IgnoreOptions {
    fn default() -> Self {
        Self {
            ignore_threads: true,
            ignore_other_users: true,
            paths: vec![],
        }
    }
}

impl PartialEq for IgnoreOptions {
    fn eq(&self, other: &Self) -> bool {
        let mut eq = self.ignore_threads == other.ignore_threads
            && self.ignore_other_users == other.ignore_other_users
            && self.paths.len() == other.paths.len();
        if eq {
            eq = self.paths.iter().map(|r| r.as_str()).collect::<Vec<&str>>()
                == other
                    .paths
                    .iter()
                    .map(|r| r.as_str())
                    .collect::<Vec<&str>>()
        }
        eq
    }
}

impl Eq for IgnoreOptions {}

pub(super) struct IgnoredProcessesFilter<'a> {
    opt: &'a IgnoreOptions,
    current_user_id: &'a Uid,
}

impl<'a> IgnoredProcessesFilter<'a> {
    pub fn new(opt: &'a IgnoreOptions, current_user_id: &'a Uid) -> Self {
        Self {
            opt,
            current_user_id,
        }
    }

    pub fn accept(&self, prc: &impl ProcessInfo) -> bool {
        if self.opt.ignore_threads && prc.is_thread() {
            return false;
        }
        if self.opt.ignore_other_users && prc.user_id() != Some(self.current_user_id) {
            return false;
        }
        if !self.opt.paths.is_empty() && prc.cmd_path().is_some() {
            let path = prc.cmd_path().unwrap();
            return !self.opt.paths.iter().any(|regex| regex.is_match(path));
        }
        true
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

        let filter = QueryFilter::new("~fOO");
        assert_eq!(filter.search_by, SearchBy::Everywhere);
        assert_eq!(filter.query, "foo");

        let filter = QueryFilter::new("!1234");
        assert_eq!(filter.search_by, SearchBy::Pid);
        assert_eq!(filter.query, "1234");

        let filter = QueryFilter::new("@1234");
        assert_eq!(filter.search_by, SearchBy::ProcessFamily);
        assert_eq!(filter.query, "1234");

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
        assert_fuzzy_match(filter.accept(&process, None), MatchedBy::Cmd);

        process.cmd = "test".to_string();
        assert_fuzzy_match(filter.accept(&process, None), MatchedBy::Cmd);

        process.cmd = "TEST".to_string();
        assert_fuzzy_match(filter.accept(&process, None), MatchedBy::Cmd);

        process.cmd = "Testificator".to_string();
        assert_fuzzy_match(filter.accept(&process, None), MatchedBy::Cmd);

        process.cmd = "online_TESTER".to_string();
        assert_fuzzy_match(filter.accept(&process, None), MatchedBy::Cmd);
        process.cmd = "xxx".to_string();
        assert_eq!(filter.accept(&process, None), None);
    }

    #[test]
    fn query_filter_search_by_path() {
        let filter = QueryFilter::new("/test");
        let mut process = MockProcessInfo {
            cmd_path: Some("/TeSt".to_string()),
            ..Default::default()
        };
        assert_fuzzy_match(filter.accept(&process, None), MatchedBy::Path);

        // tests that fuzzy search works
        process.cmd_path = Some("/taest".to_string());
        assert_fuzzy_match(filter.accept(&process, None), MatchedBy::Path);

        process.cmd_path = Some("/test".to_string());
        assert_fuzzy_match(filter.accept(&process, None), MatchedBy::Path);

        process.cmd_path = Some("/TEST".to_string());
        assert_fuzzy_match(filter.accept(&process, None), MatchedBy::Path);

        process.cmd_path = Some("/testing_dir".to_string());
        assert_fuzzy_match(filter.accept(&process, None), MatchedBy::Path);

        process.cmd_path = Some("/cargo/tests".to_string());
        assert_fuzzy_match(filter.accept(&process, None), MatchedBy::Path);

        process.cmd_path = Some("/xxx".to_string());
        assert_eq!(filter.accept(&process, None), None);

        // '/' accepts all non empty paths
        let filter = QueryFilter::new("/");
        process.cmd_path = Some("/xxx".to_string());
        assert_existence_match(filter.accept(&process, None), MatchedBy::Path);
        process.cmd_path = None;
        assert_eq!(filter.accept(&process, None), None);
    }

    #[test]
    fn query_filter_search_by_args() {
        let filter = QueryFilter::new("-test");
        let mut process = MockProcessInfo::default();

        process = process.with_args(&["-TeSt"]);
        assert_fuzzy_match(filter.accept(&process, None), MatchedBy::Args);

        process = process.with_args(&["-test"]);
        assert_fuzzy_match(filter.accept(&process, None), MatchedBy::Args);

        process = process.with_args(&["-TEST"]);
        assert_fuzzy_match(filter.accept(&process, None), MatchedBy::Args);

        process = process.with_args(&["arg1, arg2, --testifier"]);
        assert_fuzzy_match(filter.accept(&process, None), MatchedBy::Args);

        process = process.with_args(&["testimony"]);
        assert_fuzzy_match(filter.accept(&process, None), MatchedBy::Args);

        process = process.with_args(&["-xxx"]);
        assert_eq!(filter.accept(&process, None), None);

        // '-' accepts all non empty args
        let filter = QueryFilter::new("-");
        process = process.with_args(&["-arg"]);
        assert_existence_match(filter.accept(&process, None), MatchedBy::Args);
        process = process.with_args(&[]);
        assert_eq!(filter.accept(&process, None), None);
    }

    #[test]
    fn query_filter_search_by_args_ignores_cmd_in_args() {
        let filter = QueryFilter::new("-test");
        let process = MockProcessInfo {
            cmd: "test".into(),
            args: vec!["-test".into(), "-xxx".into()],
            ..Default::default()
        };
        assert_eq!(filter.accept(&process, None), None);
    }

    #[test]
    fn query_filter_search_by_port() {
        let filter = QueryFilter::new(":12");
        let process = MockProcessInfo::default();

        assert_fuzzy_match(filter.accept(&process, Some("1234")), MatchedBy::Port);

        assert_fuzzy_match(filter.accept(&process, Some("3312")), MatchedBy::Port);

        assert_fuzzy_match(filter.accept(&process, Some("5125")), MatchedBy::Port);

        assert_fuzzy_match(
            filter.accept(&process, Some("1111, 2222, 1234")),
            MatchedBy::Port,
        );

        assert_eq!(filter.accept(&process, Some("7777")), None);

        //':' accepts all non empty ports
        let filter = QueryFilter::new(":");
        assert_existence_match(filter.accept(&process, Some("5125")), MatchedBy::Port);
        assert_eq!(filter.accept(&process, None), None);
    }

    #[test]
    fn query_filter_search_by_pid() {
        let filter = QueryFilter::new("!1234");
        let mut process = MockProcessInfo {
            pid: 1234,
            ..Default::default()
        };

        assert_exact_match(filter.accept(&process, None), MatchedBy::Pid);
        process.pid = 12345;
        assert_eq!(filter.accept(&process, None), None);
    }

    #[test]
    fn query_filter_search_by_process_family() {
        let filter = QueryFilter::new("@1234");
        let mut process = MockProcessInfo {
            pid: 1234,
            ..Default::default()
        };
        assert_exact_match(filter.accept(&process, None), MatchedBy::Pid);

        process.pid = 555;
        assert_eq!(filter.accept(&process, None), None);

        process.parent_pid = Some(1234);
        assert_exact_match(filter.accept(&process, None), MatchedBy::ParentPid);

        process.parent_pid = Some(555);
        assert_eq!(filter.accept(&process, None), None);

        process.parent_pid = None;
        assert_eq!(filter.accept(&process, None), None);
    }

    #[test]
    fn query_filter_search_everywhere() {
        let filter = QueryFilter::new("~test");
        let process = MockProcessInfo {
            cmd: "TEST".into(),
            ..Default::default()
        };
        assert_fuzzy_match(filter.accept(&process, None), MatchedBy::Cmd);

        let process = MockProcessInfo {
            cmd_path: Some("/tEsT".into()),
            ..Default::default()
        };
        assert_fuzzy_match(filter.accept(&process, None), MatchedBy::Path);

        let process = MockProcessInfo {
            args: vec!["-TeSt".into()],
            ..Default::default()
        };
        assert_fuzzy_match(filter.accept(&process, None), MatchedBy::Args);

        let process = MockProcessInfo::default();

        let filter = QueryFilter::new("~80");
        assert_fuzzy_match(filter.accept(&process, Some("8080")), MatchedBy::Port);

        let filter = QueryFilter::new("~any");
        let process = MockProcessInfo {
            cmd: "xxx".into(),
            cmd_path: Some("/xxx".into()),
            args: vec!["-xxx".into()],
            ..Default::default()
        };
        assert_eq!(filter.accept(&process, Some("1234")), None);
    }

    #[test]
    fn query_filter_search_by_none() {
        let filter = QueryFilter::new("");
        let mut process = MockProcessInfo::default();
        assert_existence_match(filter.accept(&process, None), MatchedBy::ProcessExistence);

        process.cmd = "TeSt".to_string();
        assert_existence_match(filter.accept(&process, None), MatchedBy::ProcessExistence);

        process.cmd_path = Some("/TeSt".to_string());
        assert_existence_match(filter.accept(&process, None), MatchedBy::ProcessExistence);

        process = process.with_args(&["-TeSt"]);
        assert_existence_match(filter.accept(&process, None), MatchedBy::ProcessExistence);

        assert_existence_match(
            filter.accept(&process, Some("1234")),
            MatchedBy::ProcessExistence,
        );
    }

    #[test]
    fn ignored_filter_should_ignore_thread_processes() {
        let current_user_id = Uid::from_str("1").unwrap();
        let ignore = IgnoreOptions {
            ignore_threads: true,
            ..Default::default()
        };
        let filter = IgnoredProcessesFilter::new(&ignore, &current_user_id);
        let prc = MockProcessInfo {
            is_thread: true,
            ..Default::default()
        };

        assert!(!filter.accept(&prc));
    }

    #[test]
    fn ignored_filter_should_accept_threads_processes() {
        let current_user_id = Uid::from_str("1").unwrap();
        let ignore = IgnoreOptions {
            ignore_threads: false,
            ..Default::default()
        };
        let filter = IgnoredProcessesFilter::new(&ignore, &current_user_id);
        let prc = MockProcessInfo {
            is_thread: true,
            ..Default::default()
        };

        assert!(filter.accept(&prc));
    }

    #[test]
    fn ignored_filter_should_accept_only_current_user_processes() {
        let current_user_id = Uid::from_str("1000").unwrap();
        let ignore = IgnoreOptions {
            ignore_other_users: true,
            ..Default::default()
        };
        let filter = IgnoredProcessesFilter::new(&ignore, &current_user_id);
        let mut prc = MockProcessInfo {
            user_id: current_user_id.clone(),
            ..Default::default()
        };
        assert!(filter.accept(&prc));

        prc.user_id = Uid::from_str("1001").unwrap();
        assert!(!filter.accept(&prc));
    }

    #[test]
    fn ignored_filter_should_accept_other_users_processes() {
        let current_user_id = Uid::from_str("1000").unwrap();
        let ignore = IgnoreOptions {
            ignore_other_users: false,
            ..Default::default()
        };
        let filter = IgnoredProcessesFilter::new(&ignore, &current_user_id);
        let mut prc = MockProcessInfo {
            user_id: current_user_id.clone(),
            ..Default::default()
        };
        assert!(filter.accept(&prc));

        prc.user_id = Uid::from_str("1001").unwrap();
        assert!(filter.accept(&prc));
    }

    #[test]
    fn ignored_filter_should_ignore_processes_with_paths() {
        let current_user_id = Uid::from_str("1").unwrap();
        let ignore = IgnoreOptions {
            paths: vec![
                Regex::new("/usr/bin/*").unwrap(),
                Regex::new("/bin/*").unwrap(),
            ],
            ..Default::default()
        };
        let filter = IgnoredProcessesFilter::new(&ignore, &current_user_id);
        let prc = MockProcessInfo {
            cmd_path: Some("/usr/bin/exe".into()),
            ..Default::default()
        };
        assert!(!filter.accept(&prc));
        let prc = MockProcessInfo {
            cmd_path: Some("/bin/fireifox".into()),
            ..Default::default()
        };
        assert!(!filter.accept(&prc));
    }

    #[test]
    fn ignored_filter_should_accept_processes_that_does_not_match_path() {
        let current_user_id = Uid::from_str("1").unwrap();
        let ignore = IgnoreOptions {
            paths: vec![Regex::new("/usr/bin/*").unwrap()],
            ..Default::default()
        };
        let filter = IgnoredProcessesFilter::new(&ignore, &current_user_id);
        let prc = MockProcessInfo {
            cmd_path: Some("/bin/exe".into()),
            ..Default::default()
        };
        assert!(filter.accept(&prc));
    }

    fn assert_fuzzy_match(match_data: Option<MatchData>, expected_matched_by: MatchedBy) {
        let matched = ensure_matched_by(match_data, expected_matched_by);
        assert!(matches!(matched.match_type, MatchType::Fuzzy { .. }));
    }

    fn ensure_matched_by(
        match_data: Option<MatchData>,
        expected_matched_by: MatchedBy,
    ) -> MatchData {
        assert!(match_data.is_some());
        let match_data = match_data.unwrap();
        assert_eq!(match_data.matched_by, expected_matched_by);
        match_data
    }

    fn assert_exact_match(match_data: Option<MatchData>, expected_matched_by: MatchedBy) {
        let matched = ensure_matched_by(match_data, expected_matched_by);
        assert!(matches!(matched.match_type, MatchType::Exact));
    }

    fn assert_existence_match(match_data: Option<MatchData>, expected_matched_by: MatchedBy) {
        let matched = ensure_matched_by(match_data, expected_matched_by);
        assert!(matches!(matched.match_type, MatchType::Exists));
    }
}
