use crate::track::Track;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::result::Result;
use walkdir::WalkDir;

const SUPPORTED_CODECS: [&str; 5] = ["mp3", "mp4", "wav", "ogg", "flac"];

#[derive(Default)]
pub struct Library {
    pub tracks: Vec<Track>,
    pub total_duration: u64,
}

impl Library {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.tracks.len()
    }

    pub fn load(&mut self, path: &Path) {
        self.tracks = Self::scan(path)
            .iter()
            .filter_map(Track::from_path)
            .collect::<Vec<Track>>();

        self.total_duration = self.tracks.iter().map(|t| t.duration).sum();
    }

    fn scan(path: &Path) -> Vec<PathBuf> {
        WalkDir::new(path)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| !e.path().is_dir() && Self::is_supported_codec(e.path()))
            .map(|e| e.path().to_path_buf())
            .collect()
    }

    fn is_supported_codec(file: &Path) -> bool {
        file.extension()
            .and_then(OsStr::to_str)
            .is_some_and(|extension| SUPPORTED_CODECS.contains(&extension))
    }
}
