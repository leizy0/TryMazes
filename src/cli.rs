use std::{
    fs::File,
    io::{BufReader, BufWriter, Write},
    path::{Path, PathBuf},
};

use anyhow::Error as AnyError;
use clap::{Args, Subcommand};
use serde::{Serialize, de::DeserializeOwned};
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

const DEF_SHOW_WND_WIDTH: usize = 800;
const DEF_SHOW_WND_HEIGHT: usize = 600;

#[derive(Debug, Clone, Subcommand)]
pub enum GeneralMazeAction {
    /// Show maze picture in GUI
    Show {
        /// Width of window for showing maze
        #[arg(long, default_value_t = DEF_SHOW_WND_WIDTH)]
        wnd_width: usize,
        /// Height of window for showing maze
        #[arg(long, default_value_t = DEF_SHOW_WND_HEIGHT)]
        wnd_height: usize,
    },
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

pub fn load_from_json<P: AsRef<Path>, M: DeserializeOwned>(path: P) -> Result<M, AnyError> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let maze = serde_json::from_reader(reader)?;
    Ok(maze)
}

pub fn save_to_json<P: AsRef<Path>, M: Serialize>(path: P, maze: &M) -> Result<(), AnyError> {
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer(writer, &maze)?;
    Ok(())
}
