use crate::track::TrackId;

#[derive(Default)]
pub struct Filter {
    query: String,
    result: Vec<TrackId>,
}

impl Filter {
    /// Applies a predicate function `f` to `track_ids` slice.
    /// Stores the raw query and filtering result in `self`.
    pub fn apply<F>(&mut self, query: &str, track_ids: &[TrackId], f: F)
    where
        F: Fn(TrackId) -> bool,
    {
        self.query = query.to_string();
        self.result = if query.is_empty() {
            track_ids.to_vec()
        } else {
            track_ids.iter().filter(|&&id| f(id)).copied().collect()
        };
    }

    /// Returns filtering result.
    pub fn result(&self) -> &[TrackId] {
        &self.result
    }

    /// Returns true if `query` is not equal to raw `self.query`.
    pub fn is_stale(&self, query: &str) -> bool {
        self.query != query
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_with_matches() {
        let mut filter = Filter::default();
        let ids = vec![1, 2, 3, 4, 5, 6];
        filter.apply("survival", &ids, |id| id % 2 == 0);
        assert_eq!(filter.result(), &[2, 4, 6]);
    }

    #[test]
    fn filters_with_no_matches() {
        let mut filter = Filter::default();
        let ids = vec![1, 2, 3, 4, 5, 6];
        filter.apply("survival", &ids, |id| id > 10);
        assert_eq!(filter.result(), &[]);
    }

    #[test]
    fn filters_with_empty_query() {
        let mut filter = Filter::default();
        let ids = vec![1, 2, 3, 4, 5, 6];
        filter.apply("", &ids, |id| id % 2 == 0);
        assert_eq!(filter.result(), &ids);
    }

    #[test]
    fn filters_with_empty_slice() {
        let mut filter = Filter::default();
        let ids = vec![];
        filter.apply("true", &ids, |_| true);
        assert_eq!(filter.result(), &[]);
    }

    #[test]
    fn checks_if_stale() {
        let mut filter = Filter::default();
        let ids = vec![1, 2, 3, 4, 5, 6];
        filter.apply("survival", &ids, |id| id % 2 == 0);
        assert_eq!(filter.is_stale("survival"), false);
        filter.apply("SURVIVAL", &ids, |id| id % 2 == 0);
        assert_eq!(filter.is_stale("survival"), true);
    }
}
