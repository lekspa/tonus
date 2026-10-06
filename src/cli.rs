use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Sets path to music directory
    #[arg(short, long, value_name = "DIR")]
    pub music_directory: Option<PathBuf>,
}
