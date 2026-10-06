mod app;
mod cli;
mod colors;
mod config;
mod format;
mod library;
mod player;
mod search;
mod timer;
mod track;
mod truncate;
mod ui;

use clap::Parser;
use color_eyre::eyre::Result;

use crate::{app::App, cli::Cli};

fn main() -> Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();
    let config = config::load(cli.clone())?;
    ratatui::run(|terminal| App::new(config, cli)?.run(terminal))
}
