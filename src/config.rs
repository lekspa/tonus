use color_eyre::eyre::{self, Context};
use serde::Deserialize;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

#[derive(Default, Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub music_directory: Option<PathBuf>,
}

pub fn load() -> eyre::Result<Config> {
    let config_path = config_file_path();

    let mut config = config_path
        .as_deref()
        .map(load_from_disk)
        .transpose()?
        .unwrap_or_default()
        .unwrap_or_default();

    let system = dirs::audio_dir().filter(|d| d.is_dir());

    config.music_directory = config.music_directory.or(system);

    Ok(config)
}

fn config_file_path() -> Option<PathBuf> {
    dirs::config_dir().map(|path| path.join("sonus").join("config.toml"))
}

fn load_from_disk(path: &Path) -> eyre::Result<Option<Config>> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
        Err(e) => {
            return Err(e).wrap_err(format!("cannot read {}", path.display()));
        }
    };
    let config = toml::from_str::<Config>(&content)
        .wrap_err(format!("failed to parse {}", path.display()))?;
    Ok(Some(config))
}
