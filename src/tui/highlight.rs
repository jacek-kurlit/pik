use itertools::Itertools;
use ratatui::{
    style::Style,
    text::{Line, Span},
};
use regions::chunk_into_limitted_matched_regions;

use crate::processes::MatchType;

mod regions;

pub const TRUNCATED_STR: &str = "..";

#[derive(Debug, PartialEq)]
pub struct Region {
    matched: bool,
    text: String,
}

pub fn highlight_text<'a>(
    text: &'a str,
    match_type: &MatchType,
    highlighted_style: Style,
    default_style: Style,
    max_len: usize,
) -> Line<'a> {
    match match_type {
        MatchType::Exact => styled_truncated_line(text, highlighted_style, max_len),
        MatchType::Fuzzy {
            score: _,
            positions,
        } => highlight_fuzzy(text, positions, highlighted_style, default_style, max_len),
        MatchType::Exists => styled_truncated_line(text, default_style, max_len),
    }
}

fn styled_truncated_line(text: &str, style: Style, max_len: usize) -> Line {
    if text.len() > max_len {
        Line::from(vec![
            Span::styled(text.chars().take(max_len).join(""), style),
            Span::raw(TRUNCATED_STR),
        ])
    } else {
        Line::from(Span::styled(text, style))
    }
}

impl Region {
    pub fn new<T: Into<String>>(matched: bool, text: T) -> Self {
        Self {
            matched,
            text: text.into(),
        }
    }

    #[allow(dead_code)]
    pub fn matched<T: Into<String>>(text: T) -> Self {
        Self::new(true, text)
    }

    #[allow(dead_code)]
    pub fn unmatched<T: Into<String>>(text: T) -> Self {
        Self::new(false, text)
    }

    pub fn truncated_placeholder() -> Self {
        Self::new(false, TRUNCATED_STR)
    }
}

fn highlight_fuzzy<'a>(
    text: &'a str,
    positions: &[usize],
    highlighted_style: Style,
    default_style: Style,
    max_len: usize,
) -> Line<'a> {
    let spans = chunk_into_limitted_matched_regions(text, positions, max_len)
        .into_iter()
        .map(|region| {
            let style = match region.matched {
                true => highlighted_style,
                false => default_style,
            };
            Span::styled(region.text, style)
        })
        .collect_vec();

    Line::from(spans)
}

#[cfg(test)]
mod tests {
    use std::vec;

    use ratatui::{
        style::{Color, Style},
        text::Span,
    };

    use crate::tui::highlight::{highlight_fuzzy, TRUNCATED_STR};

    use super::styled_truncated_line;

    #[test]
    fn no_positions_returns_line() {
        let highlighted_style = Style::new().bg(Color::Yellow).fg(Color::Black);
        let default_style = Style::default();
        let line = highlight_fuzzy("abc", &[], highlighted_style, default_style, 10);
        assert_eq!(line.spans, vec![Span::styled("abc", default_style)]);
    }

    #[test]
    fn positions_returns_highlighted_line() {
        let highlighted_style = Style::new().bg(Color::Yellow).fg(Color::Black);
        let default_style = Style::default();
        let line = highlight_fuzzy("abcdef", &[0, 2], highlighted_style, default_style, 10);
        assert_eq!(
            line.spans,
            vec![
                Span::styled("a", highlighted_style),
                Span::styled("b", default_style),
                Span::styled("c", highlighted_style),
                Span::styled("def", default_style)
            ]
        );
    }

    #[test]
    fn highlights_utf16_text() {
        let highlighted_style = Style::new().bg(Color::Yellow).fg(Color::Black);
        let default_style = Style::default();
        let line = highlight_fuzzy(
            "hello, y̆world",
            &[5, 6, 7, 8, 9],
            highlighted_style,
            default_style,
            20,
        );
        assert_eq!(
            line.spans,
            vec![
                Span::styled("hello", default_style),
                Span::styled(", y̆w", highlighted_style),
                Span::styled("orld", default_style),
            ]
        );
    }

    #[test]
    fn continous_positions_are_merged() {
        let highlighted_style = Style::new().bg(Color::Yellow).fg(Color::Black);
        let default_style = Style::default();
        let line = highlight_fuzzy(
            "abcdefghi",
            &[0, 1, 2, 5, 6, 7],
            highlighted_style,
            default_style,
            10,
        );
        assert_eq!(
            line.spans,
            vec![
                Span::styled("abc", highlighted_style),
                Span::styled("de", default_style),
                Span::styled("fgh", highlighted_style),
                Span::styled("i", default_style)
            ]
        );
    }

    #[test]
    fn should_truncate_too_long_line() {
        let highlighted_style = Style::new().bg(Color::Yellow).fg(Color::Black);
        let default_style = Style::default();

        let line = highlight_fuzzy("abcdefghi", &[0, 1, 2], highlighted_style, default_style, 5);
        assert_eq!(
            line.spans,
            vec![
                Span::styled("abc", highlighted_style),
                Span::styled("de", default_style),
                Span::styled(TRUNCATED_STR, default_style),
            ]
        );

        let line = highlight_fuzzy(
            "abcdefghi",
            &[0, 1, 5, 6, 7],
            highlighted_style,
            default_style,
            5,
        );
        assert_eq!(
            line.spans,
            vec![
                Span::styled("ab", highlighted_style),
                Span::styled("cde", default_style),
                Span::styled(TRUNCATED_STR, default_style),
            ]
        );

        let line = highlight_fuzzy("abc", &[0, 1, 2], highlighted_style, default_style, 3);
        assert_eq!(line.spans, vec![Span::styled("abc", highlighted_style),]);

        let line = highlight_fuzzy("abcd", &[0, 1, 2], highlighted_style, default_style, 3);
        assert_eq!(
            line.spans,
            vec![
                Span::styled("abc", highlighted_style),
                Span::styled(TRUNCATED_STR, default_style)
            ]
        );

        let line = highlight_fuzzy("abcdxxxxx", &[2, 3], highlighted_style, default_style, 4);
        assert_eq!(
            line.spans,
            vec![
                Span::styled(TRUNCATED_STR, default_style),
                Span::styled("cd", highlighted_style),
                Span::styled("xx", default_style),
                Span::styled(TRUNCATED_STR, default_style)
            ]
        );

        let line = highlight_fuzzy(
            "xxxxxxxabcde",
            &[9, 10],
            highlighted_style,
            default_style,
            5,
        );
        assert_eq!(
            line.spans,
            vec![
                Span::styled(TRUNCATED_STR, default_style),
                Span::styled("ab", default_style),
                Span::styled("cd", highlighted_style),
                Span::styled("e", default_style),
            ]
        );

        //truncates matched area to fit max len
        let line = highlight_fuzzy("xxxxabcxxxxx", &[4, 5], highlighted_style, default_style, 2);
        assert_eq!(
            line.spans,
            vec![
                Span::styled(TRUNCATED_STR, default_style),
                Span::styled("ab", highlighted_style),
                Span::styled(TRUNCATED_STR, default_style)
            ]
        );

        //always prefer matched area
        let line = highlight_fuzzy("xxxxabcxxxxx", &[5, 6], highlighted_style, default_style, 2);
        assert_eq!(
            line.spans,
            vec![
                Span::styled(TRUNCATED_STR, default_style),
                Span::styled("bc", highlighted_style),
                Span::styled(TRUNCATED_STR, default_style)
            ]
        );

        let line = highlight_fuzzy(
            "abcxxxxxdef",
            &[0, 1, 2, 8, 9, 10],
            highlighted_style,
            default_style,
            5,
        );
        assert_eq!(
            line.spans,
            vec![
                Span::styled("abc", highlighted_style),
                Span::styled("xx", default_style),
                Span::styled(TRUNCATED_STR, default_style)
            ]
        );
    }

    #[test]
    fn should_truncate_styled_line() {
        let style = Style::new().bg(Color::Yellow).fg(Color::Black);
        let line = styled_truncated_line("hello, y̆world", style, 10);
        assert_eq!(
            line.spans,
            vec![Span::styled("hello, y̆w", style), Span::raw(TRUNCATED_STR)]
        )
    }
}
