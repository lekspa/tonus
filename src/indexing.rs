/// Returns the next index from `current` position wrapping around the `len`.
pub fn next_index(current: Option<usize>, len: usize) -> usize {
    debug_assert!(len > 0, "caller must guard against an empty slice");
    current.map_or(0, |index| (index + 1) % len)
}

/// Returns the previous index from `current` position wrapping around the `len`.
pub fn prev_index(current: Option<usize>, len: usize) -> usize {
    debug_assert!(len > 0, "caller must guard against an empty slice");
    current.map_or_else(|| len.saturating_sub(1), |index| (index + len - 1) % len)
}

/// Returns index of `element` in a given `slice`.
pub fn index_of<T: PartialEq>(element: Option<T>, slice: &[T]) -> Option<usize> {
    element.and_then(|e| slice.iter().position(|x| *x == e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_next_index() {
        assert_eq!(next_index(Some(0), 3), 1);
        assert_eq!(next_index(Some(1), 3), 2);
        assert_eq!(next_index(Some(2), 3), 0);
        assert_eq!(next_index(None, 3), 0);
    }

    #[test]
    fn returns_prev_index() {
        assert_eq!(prev_index(Some(2), 3), 1);
        assert_eq!(prev_index(Some(1), 3), 0);
        assert_eq!(prev_index(Some(0), 3), 2);
        assert_eq!(prev_index(None, 3), 2);
    }

    #[test]
    fn returns_index_of_element() {
        let slice = vec![1, 2, 3, 4, 5, 6];
        assert_eq!(index_of(Some(2), &slice), Some(1));
        assert_eq!(index_of(Some(6), &slice), Some(5));
        assert_eq!(index_of(Some(4), &vec![]), None);
        assert_eq!(index_of(None, &slice), None);
    }
}
