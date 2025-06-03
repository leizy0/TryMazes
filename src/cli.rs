use std::path::PathBuf;

use clap::{Args, Subcommand};
use thiserror::Error;

use crate::show::SavePictureFormat;

#[derive(Debug, Clone, Error)]
pub enum Error {
    #[error("Algorithm {0} doesn't support mask.")]
    NotSupportMask(String),
}

#[derive(Debug, Clone, Args)]
pub struct GeneralMazeLoadArgs {
    /// Path to load maze(saved as json format before)
    pub load_path: PathBuf,
    /// What to do with loaded maze
    #[command(subcommand)]
    pub action: GeneralMazeAction,
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
        action: GeneralMazeAction,
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
        action: GeneralMazeAction,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum GeneralMazeAction {
    /// Show maze picture in GUI
    Show,
    /// Show maze picture in file
    Save {
        /// Save to a picture
        #[arg(long, group = "save category", requires = "picture format")]
        picture: bool,
        /// Save to json
        #[arg(long, group = "save category")]
        json: bool,
        /// Path to save maze picture
        path: PathBuf,
        /// Format to save maze picture
        #[arg(short, long, group = "picture format")]
        format: Option<SavePictureFormat>,
    },
}
