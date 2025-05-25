use std::path::PathBuf;

use clap::Subcommand;
use thiserror::Error;

use crate::show::SavePictureFormat;

#[derive(Debug, Clone, Error)]
pub enum Error {
    #[error("Algorithm {0} doesn't support mask.")]
    NotSupportMask(String),
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
