pub struct HighlighSpliterator<'a> {
    text: &'a str,
    positions: Vec<usize>,
    previous_position: usize,
    current_index: usize,
}

impl<'a> HighlighSpliterator<'a> {
    pub fn new<'b>(text: &'a str, positions: &'b [usize]) -> Self {
        Self {
            text,
            positions: prepare_positions(positions, text.len()),
            previous_position: 0,
            current_index: 0,
        }
    }
}

impl<'a> Iterator for HighlighSpliterator<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_index == self.positions.len() {
            return None;
        }
        let position = self.positions[self.current_index];
        let range = &self.text[self.previous_position..position];
        self.previous_position = position;
        self.current_index += 1;
        Some(range)
    }
}

fn prepare_positions(positions: &[usize], max: usize) -> Vec<usize> {
    if positions.is_empty() {
        return vec![max];
    }

    let mut result = Vec::new();
    let mut current = 0;
    if positions[0] != 0 {
        current = positions[0];
        result.push(current);
    }

    for index in positions.iter().chain(vec![&max]) {
        if current == *index {
            current += 1;
            continue;
        }
        result.push(current);
        result.push(*index);
        current = *index + 1;
    }
    let last = result.last().unwrap_or(&0);
    if *last != max {
        result.push(max);
    }
    result
}

#[cfg(test)]
mod tests {

    use super::{prepare_positions, HighlighSpliterator};

    #[test]
    fn test_prepare_positions() {
        assert_eq!(prepare_positions(&[], 0), vec![0]);
        assert_eq!(prepare_positions(&[], 5), vec![5]);
        assert_eq!(prepare_positions(&[0, 2], 4), vec![1, 2, 3, 4]);
        assert_eq!(prepare_positions(&[0], 4), vec![1, 4]);
        assert_eq!(prepare_positions(&[2], 4), vec![2, 3, 4]);
        assert_eq!(prepare_positions(&[3], 4), vec![3, 4]);
        assert_eq!(prepare_positions(&[0, 2, 3], 4), vec![1, 2, 4]);
        assert_eq!(prepare_positions(&[0, 2, 3], 5), vec![1, 2, 4, 5]);
        assert_eq!(
            prepare_positions(&[0, 2, 3, 7, 8, 10, 11, 15], 20),
            vec![1, 2, 4, 7, 9, 10, 12, 15, 16, 20]
        );
        assert_eq!(prepare_positions(&[0, 1, 2], 5), vec![3, 5]);
        assert_eq!(prepare_positions(&[1, 2], 10), vec![1, 3, 10]);
        assert_eq!(prepare_positions(&[2], 10), vec![2, 3, 10]);
        assert_eq!(
            prepare_positions(&[2, 3, 4, 6, 7, 8, 10], 20),
            vec![2, 5, 6, 9, 10, 11, 20]
        );
        assert_eq!(
            prepare_positions(&[0, 1, 2, 5, 6, 7], 10),
            vec![3, 5, 8, 10]
        );
        assert_eq!(prepare_positions(&[0, 1, 2], 3), vec![3]);
        assert_eq!(prepare_positions(&[0, 2, 4, 5], 6), vec![1, 2, 3, 4, 6]);
        assert_eq!(
            prepare_positions(&[0, 2, 4, 6], 7),
            vec![1, 2, 3, 4, 5, 6, 7]
        );
        assert_eq!(
            prepare_positions(&[0, 2, 4, 6], 9),
            vec![1, 2, 3, 4, 5, 6, 7, 9]
        );
    }

    #[test]
    fn test_should_split_text_into_positions_chunks() {
        let mut spliterator = HighlighSpliterator::new("abcdef", &[0, 2]);
        assert_eq!(spliterator.next(), Some("a"));
        assert_eq!(spliterator.next(), Some("b"));
        assert_eq!(spliterator.next(), Some("c"));
        assert_eq!(spliterator.next(), Some("def"));
        assert_eq!(spliterator.next(), None);

        let mut spliterator = HighlighSpliterator::new("abcdef", &[0, 1, 2]);
        assert_eq!(spliterator.next(), Some("abc"));
        assert_eq!(spliterator.next(), Some("def"));
        assert_eq!(spliterator.next(), None);

        let mut spliterator = HighlighSpliterator::new("abcdefghi", &[0, 1, 2, 5, 6, 7]);
        assert_eq!(spliterator.next(), Some("abc"));
        assert_eq!(spliterator.next(), Some("de"));
        assert_eq!(spliterator.next(), Some("fgh"));
        assert_eq!(spliterator.next(), Some("i"));
        assert_eq!(spliterator.next(), None);

        let mut spliterator = HighlighSpliterator::new("abcdef", &[1, 2]);
        assert_eq!(spliterator.next(), Some("a"));
        assert_eq!(spliterator.next(), Some("bc"));
        assert_eq!(spliterator.next(), Some("def"));
        assert_eq!(spliterator.next(), None);

        let mut spliterator = HighlighSpliterator::new("abcdef", &[2]);
        assert_eq!(spliterator.next(), Some("ab"));
        assert_eq!(spliterator.next(), Some("c"));
        assert_eq!(spliterator.next(), Some("def"));
        assert_eq!(spliterator.next(), None);

        let mut spliterator = HighlighSpliterator::new("abc", &[]);
        assert_eq!(spliterator.next(), Some("abc"));
        assert_eq!(spliterator.next(), None);

        let mut spliterator = HighlighSpliterator::new("", &[]);
        assert_eq!(spliterator.next(), Some(""));
        assert_eq!(spliterator.next(), None);
    }
}
