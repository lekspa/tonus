use color_eyre::eyre::Context;
use lofty::prelude::*;
use lofty::read_from_path;
use std::path::PathBuf;

#[derive(Debug)]
pub struct TrackId(pub PathBuf);

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
            id: TrackId(path.clone()),
            path: path.clone(),
            title,
            artist,
            album,
            duration: tagged_file.properties().duration().as_secs(),
        })
    }
}
