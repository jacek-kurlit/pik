use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use sysinfo::Uid;

use super::{utils::get_process_args, MatchData, ProcessInfo};

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
    pub(super) fn accept(&self, prc: &impl ProcessInfo, ports: Option<&str>) -> MatchData {
        match self.search_by {
            SearchBy::Cmd => self.query_match_str(prc.cmd()),
            SearchBy::Path => self.query_matches_opt(prc.cmd_path()),
            SearchBy::Args => self.query_contains_vec(get_process_args(prc)),
            SearchBy::Port => self.query_matches_opt(ports),
            SearchBy::Pid => self.query_eq_u32(prc.pid()),
            SearchBy::ProcessFamily => self.query_matches_process_family(prc),
            SearchBy::Everywhere => {
                let matched = self.query_match_str(prc.cmd());
                if matched.positive_match() {
                    return matched;
                }
                let matched = self.query_matches_opt(prc.cmd_path());
                if matched.positive_match() {
                    return matched;
                }
                let matched = self.query_matches_opt(ports);
                if matched.positive_match() {
                    return matched;
                }
                let matched = self.query_contains_vec(get_process_args(prc));
                if matched.positive_match() {
                    return matched;
                }
                MatchData::none()
            }
            SearchBy::None => MatchData::perfect(),
        }
    }

    fn query_match_str(&self, s: &str) -> MatchData {
        match self.matcher.fuzzy_match(s, self.query.as_str()) {
            Some(score) => MatchData::new(score),
            None => MatchData::none(),
        }
    }

    fn query_matches_opt(&self, s: Option<&str>) -> MatchData {
        s.map(|s| self.query_match_str(s))
            .unwrap_or(MatchData::none())
    }

    fn query_contains_vec(&self, s: Vec<&str>) -> MatchData {
        if s.iter().any(|a| a.to_lowercase().contains(&self.query)) {
            MatchData::perfect()
        } else {
            MatchData::none()
        }
    }

    fn query_eq_u32(&self, s: u32) -> MatchData {
        if s.to_string() == self.query {
            MatchData::perfect()
        } else {
            MatchData::none()
        }
    }

    fn query_matches_process_family(&self, prc: &impl ProcessInfo) -> MatchData {
        if prc.pid().to_string() == self.query {
            return MatchData::perfect();
        }
        if prc
            .parent_id()
            .map(|pid| pid.to_string() == self.query)
            .unwrap_or(false)
        {
            return MatchData::perfect();
        }
        MatchData::none()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct FilterOptions {
    //NOTE: On linux threads can be listed as processes and thus needs filtering
    pub ignore_threads: bool,
    pub include_all_processes: bool,
}

impl Default for FilterOptions {
    fn default() -> Self {
        Self {
            ignore_threads: true,
            include_all_processes: false,
        }
    }
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
            if self.opt.include_all_processes {
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
        assert!(filter.accept(&process, None).positive_match());

        process.cmd = "test".to_string();
        assert!(filter.accept(&process, None).positive_match());

        process.cmd = "TEST".to_string();
        assert!(filter.accept(&process, None).positive_match());

        process.cmd = "Testificator".to_string();
        assert!(filter.accept(&process, None).positive_match());

        process.cmd = "online_TESTER".to_string();
        assert!(filter.accept(&process, None).positive_match());

        process.cmd = "xxx".to_string();
        assert!(filter.accept(&process, None).negative_match());
    }

    #[test]
    fn query_filter_search_by_path() {
        let filter = QueryFilter::new("/test");
        let mut process = MockProcessInfo {
            cmd_path: Some("/TeSt".to_string()),
            ..Default::default()
        };
        assert!(filter.accept(&process, None).positive_match());

        // tests that fuzzy search works
        process.cmd_path = Some("/taest".to_string());
        assert!(filter.accept(&process, None).positive_match());

        process.cmd_path = Some("/test".to_string());
        assert!(filter.accept(&process, None).positive_match());

        process.cmd_path = Some("/TEST".to_string());
        assert!(filter.accept(&process, None).positive_match());

        process.cmd_path = Some("/testing_dir".to_string());
        assert!(filter.accept(&process, None).positive_match());

        process.cmd_path = Some("/cargo/tests".to_string());
        assert!(filter.accept(&process, None).positive_match());

        process.cmd_path = Some("/xxx".to_string());
        assert!(filter.accept(&process, None).negative_match());
    }

    #[test]
    fn query_filter_search_by_args() {
        let filter = QueryFilter::new("-test");
        let mut process = MockProcessInfo::default();

        process = process.with_args(&["-TeSt"]);
        assert!(filter.accept(&process, None).positive_match());

        process = process.with_args(&["-test"]);
        assert!(filter.accept(&process, None).positive_match());

        process = process.with_args(&["-TEST"]);
        assert!(filter.accept(&process, None).positive_match());

        process = process.with_args(&["arg1, arg2, --testifier"]);
        assert!(filter.accept(&process, None).positive_match());

        process = process.with_args(&["testimony"]);
        assert!(filter.accept(&process, None).positive_match());

        process = process.with_args(&["-xxx"]);
        assert!(filter.accept(&process, None).negative_match());
    }

    #[test]
    fn query_filter_search_by_args_ignores_cmd_in_args() {
        let filter = QueryFilter::new("-test");
        let process = MockProcessInfo {
            cmd: "test".into(),
            args: vec!["-test".into(), "-xxx".into()],
            ..Default::default()
        };
        assert!(filter.accept(&process, None).negative_match());
    }

    #[test]
    fn query_filter_search_by_port() {
        let filter = QueryFilter::new(":12");
        let process = MockProcessInfo::default();

        assert!(filter.accept(&process, Some("1234")).positive_match());

        assert!(filter.accept(&process, Some("3312")).positive_match());

        assert!(filter.accept(&process, Some("5125")).positive_match());

        assert!(filter
            .accept(&process, Some("1111, 2222, 1234"))
            .positive_match());

        assert!(filter.accept(&process, Some("7777")).negative_match());
    }

    #[test]
    fn query_filter_search_by_pid() {
        let filter = QueryFilter::new("!1234");
        let mut process = MockProcessInfo {
            pid: 1234,
            ..Default::default()
        };

        assert!(filter.accept(&process, None).positive_match());
        process.pid = 12345;
        assert!(filter.accept(&process, None).negative_match());
    }

    #[test]
    fn query_filter_search_by_process_family() {
        let filter = QueryFilter::new("@1234");
        let mut process = MockProcessInfo {
            pid: 1234,
            ..Default::default()
        };

        assert!(filter.accept(&process, None).positive_match());
        process.pid = 555;
        assert!(filter.accept(&process, None).negative_match());

        process.parent_pid = Some(1234);
        assert!(filter.accept(&process, None).positive_match());
        process.parent_pid = Some(555);
        assert!(filter.accept(&process, None).negative_match());
        process.parent_pid = None;
        assert!(filter.accept(&process, None).negative_match());
    }

    #[test]
    fn query_filter_search_everywhere() {
        let mut filter = QueryFilter::new("~test");
        let mut process = MockProcessInfo {
            cmd: "TEST".into(),
            ..Default::default()
        };
        assert!(filter.accept(&process, None).positive_match());

        process.cmd_path = Some("/tEsT".into());
        assert!(filter.accept(&process, None).positive_match());

        process = process.with_args(&["-TeSt"]);
        assert!(filter.accept(&process, None).positive_match());

        filter = QueryFilter::new("~80");
        assert!(filter.accept(&process, Some("8080")).positive_match());

        process.cmd = "xxx".into();
        process.cmd_path = Some("/xxx".into());
        process = process.with_args(&["-xxx"]);
        assert!(filter.accept(&process, Some("1234")).negative_match());
    }

    #[test]
    fn query_filter_search_by_none() {
        let filter = QueryFilter::new("");
        let mut process = MockProcessInfo::default();
        assert!(filter.accept(&process, None).positive_match());

        process.cmd = "TeSt".to_string();
        assert!(filter.accept(&process, None).positive_match());

        process.cmd_path = Some("/TeSt".to_string());
        assert!(filter.accept(&process, None).positive_match());

        process = process.with_args(&["-TeSt"]);
        assert!(filter.accept(&process, None).positive_match());

        assert!(filter.accept(&process, Some("1234")).positive_match());
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
                include_all_processes: false,
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
                include_all_processes: true,
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
