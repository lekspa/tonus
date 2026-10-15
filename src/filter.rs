use crate::{library::Library, track::TrackId};

#[derive(Default)]
pub struct Filter {
    query: String,
    result: Vec<TrackId>,
}

impl Filter {
    pub fn result(&self) -> &[TrackId] {
        &self.result
    }

    pub fn apply(&mut self, input: &str, library: &Library) {
        let query = input.to_lowercase();

        self.result = if query.is_empty() {
            library.track_ids().to_vec()
        } else {
            library
                .track_ids()
                .iter()
                .filter(|&&id| library.get_track(id).is_some_and(|t| t.contains(&query)))
                .copied()
                .collect()
        };
        self.query = input.to_string();
    }

    pub fn is_stale(&self, query: &str) -> bool {
        self.query != query
    }
}
