mod app;
mod cli;
mod config;
mod filter;
mod indexing;
mod library;
mod logging;
mod player;
mod search;
mod timer;
mod track;
mod ui;

use clap::Parser;
use color_eyre::eyre::Result;
use tracing::error;

use crate::{app::App, cli::Cli, logging::init_tracing};

fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();
    let config = config::load(cli.clone())?;
    let _guard = match init_tracing() {
        Ok(guard) => Some(guard),
        Err(e) => {
            eprintln!("Warning: failed to initialize tracing: {e:?}");
            None
        }
    };
    let result = ratatui::run(|terminal| App::new(config, cli)?.run(terminal));

    if let Err(e) = &result {
        error!("Failed to run: {e:?}");
    }

    result
}
