use std::path::PathBuf;

use clap::{Args, Subcommand};

use crate::show::SavePictureFormat;

#[derive(Debug, Clone, Args)]
#[group(required = true, multiple = false)]
pub struct GeneralMaze2dAlgorithm {
    /// Using Aldous-Broder algorithm
    #[arg(long)]
    pub aldous_broder: bool,
    /// Using Wilson's algorithm
    #[arg(long)]
    pub wilson: bool,
    /// Using Hunt-and-Kill algorithm
    #[arg(long)]
    pub hunt_and_kill: bool,
    /// Using recursive backtracker algorithm
    #[arg(long)]
    pub recursive_backtracker: bool,
}

#[derive(Debug, Clone, Subcommand)]
pub enum MazeAction {
    /// Show maze picture in GUI
    Show,
    /// Show maze picture in file
    Save {
        /// Path to save maze picture
        path: PathBuf,
        /// Format to save maze picture
        format: SavePictureFormat,
    },
}
