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
    /// Using Kruskal's algorithm
    #[arg(long)]
    pub kruskal: bool,
    /// Using Prim's algorithm
    #[arg(long)]
    pub prim: bool,
}

#[derive(Debug, Clone, Subcommand)]
pub enum GeneralRectMazeShape {
    Size {
        /// column count of maze
        width: usize,
        /// row count of maze
        height: usize,
        /// Action to do with maze,
        #[command(subcommand)]
        action: MazeAction,
    },
    Mask {
        /// Using text mask(x or X is for not cell, other characters are for cell)
        #[arg(long, group = "mask type", required = true)]
        text: bool,
        /// Using image mask(black pixel is for not cell, other colors are for cell)
        #[arg(long, group = "mask type", required = true)]
        image: bool,
        /// Path of mask file
        path: PathBuf,
        /// Action to do with maze,
        #[command(subcommand)]
        action: MazeAction,
    },
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
