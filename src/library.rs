use crate::track::{Track, TrackId};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::result::Result;
use walkdir::WalkDir;

const SUPPORTED_CODECS: [&str; 5] = ["mp3", "mp4", "wav", "ogg", "flac"];

#[derive(Default)]
pub struct Library {
    pub tracks: HashMap<TrackId, Track>,
    pub tracks_ids: Vec<TrackId>,
    pub total_duration: u64,
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

impl Library {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.tracks.len()
    }

    pub fn load(&mut self, path: &Path) {
        self.tracks = scan(path)
            .iter()
            .filter_map(Track::from_path)
            .map(|t| (t.id, t))
            .collect::<HashMap<TrackId, Track>>();
        self.tracks_ids = self.tracks.keys().copied().collect();
        self.total_duration = self.tracks.values().map(|t| t.duration).sum();
    }
}
