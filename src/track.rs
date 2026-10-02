use color_eyre::eyre::Context;
use lofty::prelude::*;
use lofty::read_from_path;
use std::hash::Hash;
use std::hash::{DefaultHasher, Hasher};
use std::path::{Path, PathBuf};

pub type TrackId = u64;

#[derive(Debug)]
pub struct Track {
    pub id: TrackId,
    pub path: PathBuf,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: u64,
}

impl Track {
    pub fn from_path(path: &PathBuf) -> Option<Self> {
        let id = path_to_id(path);

        let tagged_file = read_from_path(path).wrap_err("failed to read track").ok()?;

        let tag = tagged_file
            .primary_tag()
            .or_else(|| tagged_file.first_tag())?;

        let fallback_title = path.file_stem().map_or_else(
            || "Unknown title".to_string(),
            |s| s.to_string_lossy().into(),
        );

        let title = tag
            .title()
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or(&fallback_title)
            .to_string();

        let artist = tag
            .artist()
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or("Unknown artist")
            .to_string();

        let album = tag
            .album()
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or("Unknown album")
            .to_string();

        Some(Self {
            id,
            path: path.clone(),
            title,
            artist,
            album,
            duration: tagged_file.properties().duration().as_secs(),
        })
    }

    pub fn contains(&self, input: &str) -> bool {
        let matched_title = self.title.to_lowercase().contains(input);
        let matched_artist = self.artist.to_lowercase().contains(input);
        let matched_album = self.album.to_lowercase().contains(input);
        matched_title || matched_artist || matched_album
    }
}

fn path_to_id(path: &Path) -> u64 {
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    hasher.finish()
}
