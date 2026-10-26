use crate::track::{Track, TrackId};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::result::Result;
use std::time::Duration;
use walkdir::WalkDir;

const SUPPORTED_CODECS: [&str; 5] = ["mp3", "mp4", "wav", "ogg", "flac"];

#[derive(Default)]
pub struct Library {
    tracks: HashMap<TrackId, Track>,
    track_ids: Vec<TrackId>,
    pub total_duration: Duration,
}

impl Library {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.tracks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty()
    }

    pub fn track_ids(&self) -> &[TrackId] {
        &self.track_ids
    }

    pub fn load(&mut self, path: &Path) {
        self.tracks = scan(path)
            .iter()
            .filter_map(Track::from_path)
            .map(|t| (t.id, t))
            .collect::<HashMap<TrackId, Track>>();
        self.track_ids = self.tracks.keys().copied().collect();
        self.total_duration = self.tracks.values().map(|t| t.duration).sum();
        self.sort();
    }

    pub fn get_track(&self, id: TrackId) -> Option<&Track> {
        self.tracks.get(&id)
    }

    /// Sorts `self.track_ids` in artist -> album -> title order
    fn sort(&mut self) {
        self.track_ids.sort_by(|a, b| {
            let track_a = &self.tracks[a];
            let track_b = &self.tracks[b];
            track_a
                .artist
                .cmp(&track_b.artist)
                .then_with(|| track_a.album.cmp(&track_b.album))
                .then_with(|| track_a.title.cmp(&track_b.title))
        });
    }
}

fn is_supported_codec(file: &Path) -> bool {
    file.extension()
        .and_then(OsStr::to_str)
        .is_some_and(|extension| SUPPORTED_CODECS.contains(&extension))
}

fn scan(path: &Path) -> Vec<PathBuf> {
    WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| !e.path().is_dir() && is_supported_codec(e.path()))
        .map(|e| e.path().to_path_buf())
        .collect()
}
