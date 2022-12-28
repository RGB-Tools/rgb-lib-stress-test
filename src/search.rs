use std::fs;

use chrono::{DateTime, FixedOffset};

use grep_matcher::Matcher;
use grep_regex::RegexMatcher;
use grep_searcher::sinks::UTF8;
use grep_searcher::Searcher;

#[derive(Clone, Debug)]
pub(crate) struct LogMatch {
    pub(crate) line: u64,
    pub(crate) timestamp: String,
}

fn get_datetime(datetime: &str) -> DateTime<FixedOffset> {
    let rfc3339 = format!("{}:00", datetime);
    DateTime::parse_from_rfc3339(&rfc3339).unwrap()
}

pub(crate) fn get_time_diff(match_pair: (LogMatch, LogMatch)) -> i64 {
    let time_begin = get_datetime(&match_pair.0.timestamp);
    let time_end = get_datetime(&match_pair.1.timestamp);
    let time = time_end - time_begin;
    time.num_seconds()
}

pub(crate) fn grep_log(log_path: &str, pattern: &str) -> Vec<LogMatch> {
    let contents = fs::read_to_string(log_path).unwrap();
    let matcher = RegexMatcher::new(pattern).unwrap();
    let mut matches: Vec<LogMatch> = vec![];
    Searcher::new()
        .search_slice(
            &matcher,
            contents.as_bytes(),
            UTF8(|lnum, line| {
                matcher.find(line.as_bytes()).unwrap().unwrap();
                matches.push(LogMatch {
                    line: lnum,
                    timestamp: line[0..26].to_string(),
                });
                Ok(true)
            }),
        )
        .unwrap();
    matches
}
