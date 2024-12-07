use itertools::Itertools;
use ratatui::{
    style::Style,
    text::{Line, Span},
};
use spliterator::HighlighSpliterator;

use crate::processes::MatchType;

mod spliterator;

const TRUNCATED_STR: &str = "..";

pub fn highlight_text<'a>(
    text: &'a str,
    match_type: &MatchType,
    highlighted_style: Style,
    default_style: Style,
    max_len: usize,
) -> Line<'a> {
    match match_type {
        MatchType::Exact => styled_truncated_line(text, highlighted_style, max_len),
        MatchType::Contains { from, to } => {
            let positions = (*from..*to).collect_vec();
            highlight_fuzzy(text, &positions, highlighted_style, default_style, max_len)
        }
        MatchType::Fuzzy {
            score: _,
            positions,
        } => highlight_fuzzy(text, positions, highlighted_style, default_style, max_len),
        MatchType::Exists => styled_truncated_line(text, default_style, max_len),
    }
}

fn styled_truncated_line(text: &str, style: Style, max_len: usize) -> Line {
    let span = if text.len() > max_len {
        Span::styled(format!("{}{}", &text[0..max_len], TRUNCATED_STR), style)
    } else {
        Span::styled(text, style)
    };
    Line::from(span)
}

fn highlight_fuzzy<'a>(
    text: &'a str,
    positions: &[usize],
    highlighted_style: Style,
    default_style: Style,
    max_len: usize,
) -> Line<'a> {
    if positions.is_empty() {
        return Line::raw(text);
    }
    let first_matched_pos = positions[0];
    let mut hightlighted = first_matched_pos == 0;
    let mut spans = Vec::new();
    let mut chars_used = 0;
    let mut spliterator = HighlighSpliterator::new(text, positions);
    // We truncated front of line to skip unmateched part
    // bacause we are lack of space to display it anyway
    let last_matched_pos = *positions.last().unwrap();
    if !hightlighted && last_matched_pos >= max_len {
        spans.push(Span::styled(TRUNCATED_STR, default_style));
        hightlighted = true;
        let unmatched_head_area = spliterator.next().unwrap_or("");
        // We want to add unmatched area to line only if we have any space left
        let matched_area_len = last_matched_pos - first_matched_pos + 1;
        let unmatched_tail_len = text
            .len()
            .saturating_sub(last_matched_pos)
            .saturating_sub(1);
        let space_left = max_len
            .saturating_sub(matched_area_len)
            .saturating_sub(unmatched_tail_len);
        if space_left > 0 {
            let truncated =
                &unmatched_head_area[unmatched_head_area.len().saturating_sub(space_left)..];
            spans.push(Span::styled(truncated, default_style));
            chars_used += truncated.len();
        }
    }
    for area in spliterator {
        let style = if hightlighted {
            highlighted_style
        } else {
            default_style
        };
        chars_used += area.len();
        if chars_used > max_len {
            let truncated_len = area.len() - (chars_used - max_len);
            if truncated_len > 0 {
                let truncated = &area[0..truncated_len];
                spans.push(Span::styled(truncated, style));
            }
            spans.push(Span::styled(TRUNCATED_STR, default_style));
            break;
        }
        spans.push(Span::styled(area, style));
        hightlighted = !hightlighted;
    }

    Line::from(spans)
}

#[cfg(test)]
mod tests {
    use ratatui::{
        style::{Color, Style},
        text::Span,
    };

    use crate::tui::highlight::TRUNCATED_STR;

    use super::highlight_fuzzy;

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
                Span::styled("ab", default_style),
                Span::styled("cd", highlighted_style),
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
}
