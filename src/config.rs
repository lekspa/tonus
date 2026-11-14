use color_eyre::eyre::Result;
use serde::Deserialize;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::cli::Cli;
use crate::ui::theme::Theme;

pub const DEFAULT_CONFIG_LOCATION: &str = "~/.config/tonus/tonus.toml";

#[derive(Default, Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default)]
    music_directory: Option<PathBuf>,
    #[serde(default)]
    theme: Theme,
}

impl Config {
    pub fn music_dir(&self) -> Option<&PathBuf> {
        self.music_directory.as_ref()
    }

    pub fn theme(&self) -> &Theme {
        &self.theme
    }
}

pub fn load(cli: Cli) -> Result<Config> {
    let mut config = load_config(config_file_path())?;
    let system = dirs::audio_dir().filter(|d| d.is_dir());

    config.music_directory = cli
        .music_directory
        .or(config.music_directory)
        .or(system)
        .map(|path| expand_tilde(&path));

    Ok(config)
}

pub fn config_file_path() -> Option<PathBuf> {
    dirs::config_dir().map(|path| path.join("tonus").join("tonus.toml"))
}

fn load_config(path: Option<PathBuf>) -> Result<Config> {
    let Some(path) = path else {
        return Ok(Config::default());
    };

    match fs::read_to_string(&path) {
        Ok(contents) => Ok(toml::from_str::<Config>(&contents)?),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(Config::default()),
        Err(e) => Err(e.into()),
    }
}

fn expand_tilde(path: &Path) -> PathBuf {
    let path_str = path.to_string_lossy();

    if let Some(rest) = path_str
        .strip_prefix('~')
        .map(|s| s.trim_start_matches('/'))
        .map(|s| s.trim_end_matches('/'))
        && let Some(home) = dirs::home_dir()
    {
        return home.join(rest);
    }

    path.to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_tilde() {
        let home = dirs::home_dir().expect("home directory should exist");

        assert_eq!(expand_tilde(Path::new("~")), home);
        assert_eq!(expand_tilde(Path::new("~/")), home);
        assert_eq!(expand_tilde(Path::new("~/Music")), home.join("Music"));
        assert_eq!(expand_tilde(Path::new("~/Music/")), home.join("Music"));
        assert_eq!(expand_tilde(Path::new("/home")), Path::new("/home"));
    }
}
