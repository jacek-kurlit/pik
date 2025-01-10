use std::collections::VecDeque;

use itertools::Itertools;

use super::Region;

pub fn chunk_into_limitted_matched_regions(
    text: &str,
    positions: &[usize],
    chars_limit: usize,
) -> VecDeque<Region> {
    if text.is_empty() {
        return VecDeque::new();
    }
    let first_pos = positions.first().copied().unwrap_or(0);
    let skip_prefix = if text.len() > chars_limit && first_pos > 0 {
        // let last_pos = positions.last().copied().unwrap_or(0);
        let matched_chars_len = text.len() - first_pos;
        let space_left = chars_limit.saturating_sub(matched_chars_len);
        first_pos.saturating_sub(space_left)
    } else {
        0
    };

    let mut items: VecDeque<Region> = text
        .chars()
        .enumerate()
        .skip(skip_prefix)
        .take(chars_limit)
        .chunk_by(|(i, _)| positions.contains(i))
        .into_iter()
        .map(|(matched, chunk)| Region::new(matched, chunk.map(|(_, char)| char).join("")))
        .collect();
    if skip_prefix > 0 {
        items.push_front(Region::truncated_placeholder());
    }
    if text.len() - skip_prefix > chars_limit {
        items.push_back(Region::truncated_placeholder());
    }
    items
}
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_not_skip_entire_unmatched_prefix() {
        let mut chunks =
            chunk_into_limitted_matched_regions("xxxxxxxabcde", &[9, 10], 5).into_iter();
        assert_eq!(chunks.next(), Some(Region::truncated_placeholder()));
        assert_eq!(chunks.next(), Some(Region::unmatched("ab")));
        assert_eq!(chunks.next(), Some(Region::matched("cd")));
        assert_eq!(chunks.next(), Some(Region::unmatched("e")));
        assert_eq!(chunks.next(), None);
    }

    #[test]
    fn should_chunk_into_untruncated_matched_regions() {
        let mut chunks = chunk_into_limitted_matched_regions("abcdef", &[0, 2], 20).into_iter();
        assert_eq!(chunks.next(), Some(Region::matched("a")));
        assert_eq!(chunks.next(), Some(Region::unmatched("b")));
        assert_eq!(chunks.next(), Some(Region::matched("c")));
        assert_eq!(chunks.next(), Some(Region::unmatched("def")));
        assert_eq!(chunks.next(), None);

        let mut chunks =
            chunk_into_limitted_matched_regions("abcdefghi", &[0, 1, 2, 5, 6, 7], 20).into_iter();
        assert_eq!(chunks.next(), Some(Region::matched("abc")));
        assert_eq!(chunks.next(), Some(Region::unmatched("de")));
        assert_eq!(chunks.next(), Some(Region::matched("fgh")));
        assert_eq!(chunks.next(), Some(Region::unmatched("i")));
        assert_eq!(chunks.next(), None);

        let mut chunks = chunk_into_limitted_matched_regions("abcdef", &[1, 2], 20).into_iter();
        assert_eq!(chunks.next(), Some(Region::unmatched("a")));
        assert_eq!(chunks.next(), Some(Region::matched("bc")));
        assert_eq!(chunks.next(), Some(Region::unmatched("def")));
        assert_eq!(chunks.next(), None);

        let mut chunks = chunk_into_limitted_matched_regions("abcdef", &[2], 20).into_iter();
        assert_eq!(chunks.next(), Some(Region::unmatched("ab")));
        assert_eq!(chunks.next(), Some(Region::matched("c")));
        assert_eq!(chunks.next(), Some(Region::unmatched("def")));
        assert_eq!(chunks.next(), None);
    }

    #[test]
    fn should_correctly_chunk_utf16() {
        let mut chunks =
            chunk_into_limitted_matched_regions("hello, y̆world", &[5, 6, 7, 8, 9], 20).into_iter();
        assert_eq!(chunks.next(), Some(Region::unmatched("hello")));
        assert_eq!(chunks.next(), Some(Region::matched(", y̆w")));
        assert_eq!(chunks.next(), Some(Region::unmatched("orld")));
        assert_eq!(chunks.next(), None);
    }

    #[test]
    fn should_truncate_too_long_text() {
        let mut chunks =
            chunk_into_limitted_matched_regions("abcdefghi", &[0, 1, 2], 6).into_iter();
        assert_eq!(chunks.next(), Some(Region::matched("abc")));
        assert_eq!(chunks.next(), Some(Region::unmatched("def")));
        assert_eq!(chunks.next(), Some(Region::truncated_placeholder()));
        assert_eq!(chunks.next(), None);
    }

    #[test]
    fn should_truncate_in_the_middle_of_matched_text() {
        let mut chunks =
            chunk_into_limitted_matched_regions("abcdefghi", &[0, 1, 2, 6], 7).into_iter();
        assert_eq!(chunks.next(), Some(Region::matched("abc")));
        assert_eq!(chunks.next(), Some(Region::unmatched("def")));
        assert_eq!(chunks.next(), Some(Region::matched("g")));
        assert_eq!(chunks.next(), Some(Region::truncated_placeholder()));
        assert_eq!(chunks.next(), None);
    }

    #[test]
    fn should_truncate_entire_matched_region() {
        let mut chunks =
            chunk_into_limitted_matched_regions("abcdef", &[0, 1, 2, 3, 4, 5], 3).into_iter();
        assert_eq!(chunks.next(), Some(Region::matched("abc")));
        assert_eq!(chunks.next(), Some(Region::truncated_placeholder()));
        assert_eq!(chunks.next(), None);
    }

    #[test]
    fn should_truncate_first_unmatchede_region() {
        let mut chunks =
            chunk_into_limitted_matched_regions("abcdefghi", &[3, 4, 5], 6).into_iter();
        assert_eq!(chunks.next(), Some(Region::truncated_placeholder()));
        assert_eq!(chunks.next(), Some(Region::matched("def")));
        assert_eq!(chunks.next(), Some(Region::unmatched("ghi")));
        assert_eq!(chunks.next(), None);
    }

    #[test]
    fn should_not_truncate_first_unmatchede_region_when_no_matched_regions() {
        let mut chunks = chunk_into_limitted_matched_regions("abcdefghi", &[], 5).into_iter();
        assert_eq!(chunks.next(), Some(Region::unmatched("abcde")));
        assert_eq!(chunks.next(), Some(Region::truncated_placeholder()));
        assert_eq!(chunks.next(), None);
    }

    #[test]
    fn should_truncate_from_both_sides() {
        let mut chunks =
            chunk_into_limitted_matched_regions("abcdefghi", &[3, 4, 5], 3).into_iter();
        assert_eq!(chunks.next(), Some(Region::truncated_placeholder()));
        assert_eq!(chunks.next(), Some(Region::matched("def")));
        assert_eq!(chunks.next(), Some(Region::truncated_placeholder()));
        assert_eq!(chunks.next(), None);
    }

    #[test]
    fn should_handle_empty_text() {
        let chunks = chunk_into_limitted_matched_regions("", &[1], 10);
        assert!(chunks.is_empty());
    }

    #[test]
    fn should_handle_empty_positions() {
        let mut chunks = chunk_into_limitted_matched_regions("abc", &[], 10).into_iter();
        assert_eq!(chunks.next(), Some(Region::unmatched("abc")));
        assert_eq!(chunks.next(), None);

        let mut chunks = chunk_into_limitted_matched_regions("abcdef", &[], 3).into_iter();
        assert_eq!(chunks.next(), Some(Region::unmatched("abc")));
        assert_eq!(chunks.next(), Some(Region::truncated_placeholder()));
        assert_eq!(chunks.next(), None);
    }

    #[test]
    fn should_truncate_mixed_matched_regions() {
        let mut chunks =
            chunk_into_limitted_matched_regions("abcdefghijkl", &[3, 4, 6, 10], 5).into_iter();
        assert_eq!(chunks.next(), Some(Region::truncated_placeholder()));
        assert_eq!(chunks.next(), Some(Region::matched("de")));
        assert_eq!(chunks.next(), Some(Region::unmatched("f")));
        assert_eq!(chunks.next(), Some(Region::matched("g")));
        assert_eq!(chunks.next(), Some(Region::unmatched("h")));
        assert_eq!(chunks.next(), Some(Region::truncated_placeholder()));
        assert_eq!(chunks.next(), None);
    }
}
