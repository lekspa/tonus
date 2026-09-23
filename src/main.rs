mod app;
mod colors;
mod config;
mod format;
mod library;
mod player;
mod timer;
mod track;
mod truncate;
mod ui;

use color_eyre::eyre::Result;

use crate::app::App;

fn main() -> Result<()> {
    color_eyre::install()?;
    let config = config::load()?;
    ratatui::run(|terminal| App::new()?.run(terminal, config))
}
