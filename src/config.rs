use color_eyre::eyre::Result;
use serde::Deserialize;
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

#[derive(Default, Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub music_directory: Option<PathBuf>,
}

pub fn load() -> Result<Config> {
    let mut config = load_config(config_file_path())?;
    let system = dirs::audio_dir().filter(|d| d.is_dir());
    config.music_directory = config.music_directory.or(system);
    Ok(config)
}

pub fn config_file_path() -> Option<PathBuf> {
    dirs::config_dir().map(|path| path.join("sonus").join("config.toml"))
}

fn load_config(path: Option<PathBuf>) -> Result<Config> {
    let Some(path) = path else {
        return Ok(Config::default());
    };

    match fs::read_to_string(&path) {
        Ok(contents) => Ok(toml::from_str::<Config>(&contents)?),
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(Config::default()),
        Err(e) => return Err(e.into()),
    }
}
