use std::{fmt::Display, fs::File, io::Write, path::PathBuf};

use anyhow::Error as AnyError;
use clap::{Args, Parser, Subcommand};
use thiserror::Error;
use try_mazes::{
    gene::{
        AldousBroderMazeGenerator, HuntAndKillMazeGenerator, RecursiveBacktrackerMazeGenerator,
        WilsonMazeGenerator,
        rect::{BTreeMazeGenerator, DiagonalDirection, RectMazeGenerator, SideWinderMazeGenerator},
    },
    maze::rect::{RectGrid, RectMask},
    show::{
        MazePicture, SavePictureFormat,
        rect::{AsciiBoxCharset, RectMazeCmdDisplay, RectMazePainter, UnicodeBoxCharset},
    },
};

fn main() -> Result<(), AnyError> {
    const DEF_WALL_THICKNESS: usize = 5;
    const DEF_CELL_WIDTH: usize = 50;

    let maze_input = MazeInputArgs::parse();
    let grid = make_grid(&maze_input)?;
    let generator = make_generator(&maze_input)?;
    let maze = generator.generate(grid);
    let painter = RectMazePainter::new(&maze, DEF_WALL_THICKNESS, DEF_CELL_WIDTH);
    let picture = MazePicture::new(&painter);
    match maze_input.action {
        MazeAction::Show(ShowArgs { ascii: true, .. }) => {
            println!("{}", RectMazeCmdDisplay(&maze, AsciiBoxCharset))
        }
        MazeAction::Show(ShowArgs { unicode: true, .. }) => {
            println!("{}", RectMazeCmdDisplay(&maze, UnicodeBoxCharset))
        }
        MazeAction::Show(ShowArgs { gui: true, .. }) => picture.show()?,
        MazeAction::Save(SaveArgs {
            ascii,
            unicode,
            path,
            ..
        }) if ascii || unicode => {
            let mut file = File::create(path)?;
            let display: &dyn Display = if ascii {
                &RectMazeCmdDisplay(&maze, AsciiBoxCharset)
            } else {
                &RectMazeCmdDisplay(&maze, UnicodeBoxCharset)
            };
            file.write_all(display.to_string().as_bytes())?;
            file.flush()?;
        }
        MazeAction::Save(SaveArgs {
            picture: true,
            pic_format: Some(format),
            path,
            ..
        }) => picture.save(path, format)?,
        _ => unreachable!(
            "Given unknown action or missing arguments of action, should be checked by clap."
        ),
    }

    Ok(())
}

#[derive(Debug, Parser)]
#[command(name = "MazeDemo", version)]
#[command(about = "Demo of maze generation and display(on command line).", long_about = None)]
#[command(flatten_help = true)]
struct MazeInputArgs {
    /// Generation algorithm
    #[command(flatten)]
    algorithm: MazeGenAlgorithm,
    /// What to do with generated maze
    #[command(subcommand)]
    action: MazeAction,
}

#[derive(Debug, Clone, Copy, Args)]
struct MazeGenAlgorithm {
    /// Using binary tree algorithm
    #[arg(long, group = "algorithm", requires = "connect direction")]
    btree: bool,
    /// Using sidewinder algorithm
    #[arg(long, group = "algorithm", requires = "connect direction")]
    sidewinder: bool,
    /// Using Aldous-Broder algorithm
    #[arg(long, group = "algorithm")]
    aldous_broder: bool,
    /// Using Wilson's algorithm
    #[arg(long, group = "algorithm")]
    wilson: bool,
    /// Using Hunt-and-Kill algorithm
    #[arg(long, group = "algorithm")]
    hunt_and_kill: bool,
    /// Using recursive backtracker algorithm
    #[arg(long, group = "algorithm")]
    recursive_backtracker: bool,
    /// Candidate directions to connect
    #[arg(short, long, group = "connect direction")]
    con_dir: Option<DiagonalDirection>,
}

#[derive(Debug, Subcommand)]
enum MazeAction {
    /// Show maze by chosen way
    Show(ShowArgs),
    /// Save maze by given settings
    Save(SaveArgs),
}

#[derive(Debug, Clone, Args)]
struct ShowArgs {
    /// Using ascii characters to display maze
    #[arg(long, group = "save category")]
    ascii: bool,
    /// Using unicode box characters to display maze
    #[arg(long, group = "save category")]
    unicode: bool,
    /// Using GUI to display maze in graphics
    #[arg(long, group = "save category")]
    gui: bool,
    /// Maze shape
    #[command(subcommand)]
    shape: MazeShape,
}

#[derive(Debug, Clone, Args)]
struct SaveArgs {
    /// Using ascii characters to paint maze
    #[arg(long, group = "save category")]
    ascii: bool,
    /// Using unicode box characters to paint maze
    #[arg(long, group = "save category")]
    unicode: bool,
    /// Using graphics to paint maze
    #[arg(long, group = "save category", requires = "picture format")]
    picture: bool,
    /// Picture file format to save
    #[arg(long, group = "picture format")]
    pic_format: Option<SavePictureFormat>,
    /// Path to save
    #[arg(long = "save-path")]
    path: PathBuf,
    /// Maze shape
    #[command(subcommand)]
    shape: MazeShape,
}

#[derive(Debug, Clone, Subcommand)]
enum MazeShape {
    Size(MazeSize),
    Mask(MazeMaskInfo),
}

#[derive(Debug, Clone, Copy, Args)]
struct MazeSize {
    /// Width of maze
    #[arg(long)]
    width: usize,
    /// Height of maze
    #[arg(long)]
    height: usize,
}

#[derive(Debug, Clone, Args)]
struct MazeMaskInfo {
    /// Mask given in text
    #[arg(long, group = "mask category", requires = "mask info")]
    text: bool,
    /// Mask given in image
    #[arg(long, group = "mask category", requires = "mask info")]
    image: bool,
    /// Mask file path
    #[arg(long = "mask-path", group = "mask info")]
    path: Option<PathBuf>,
}

#[derive(Debug, Clone, Error)]
enum Error {
    #[error("Algorithm {0} doesn't support mask.")]
    NotSupportMask(String),
}

fn make_grid(input: &MazeInputArgs) -> Result<RectGrid, AnyError> {
    match &input.action {
        MazeAction::Show(ShowArgs { shape, .. }) | MazeAction::Save(SaveArgs { shape, .. }) => {
            match shape {
                MazeShape::Size(MazeSize { width, height }) => Ok(RectGrid::new(*width, *height)),
                MazeShape::Mask(MazeMaskInfo {
                    text: true,
                    path: Some(mask_path),
                    ..
                }) => Ok(RectGrid::with_mask(&RectMask::try_from_text_file(
                    mask_path,
                )?)),
                MazeShape::Mask(MazeMaskInfo {
                    image: true,
                    path: Some(mask_path),
                    ..
                }) => Ok(RectGrid::with_mask(&RectMask::try_from_image_file(
                    mask_path,
                )?)),
                other_shape => unreachable!(
                    "Given invalid shape information({:?}), should be refused by clap.",
                    other_shape
                ),
            }
        }
    }
}

fn make_generator(input: &MazeInputArgs) -> Result<Box<dyn RectMazeGenerator>, AnyError> {
    match input.algorithm {
        MazeGenAlgorithm {
            aldous_broder: true,
            ..
        } => Ok(Box::new(AldousBroderMazeGenerator)),
        MazeGenAlgorithm { wilson: true, .. } => Ok(Box::new(WilsonMazeGenerator)),
        MazeGenAlgorithm {
            hunt_and_kill: true,
            ..
        } => Ok(Box::new(HuntAndKillMazeGenerator)),
        MazeGenAlgorithm {
            recursive_backtracker: true,
            ..
        } => Ok(Box::new(RecursiveBacktrackerMazeGenerator)),
        MazeGenAlgorithm {
            btree: true,
            con_dir: Some(diag_dir),
            ..
        }
        | MazeGenAlgorithm {
            sidewinder: true,
            con_dir: Some(diag_dir),
            ..
        } => match &input.action {
            MazeAction::Show(ShowArgs { shape, .. }) | MazeAction::Save(SaveArgs { shape, .. }) => {
                match shape {
                    MazeShape::Size(_) => {
                        if input.algorithm.btree {
                            Ok(Box::new(BTreeMazeGenerator::new(diag_dir)))
                        } else {
                            Ok(Box::new(SideWinderMazeGenerator::new(diag_dir)))
                        }
                    }
                    MazeShape::Mask(_) => {
                        if input.algorithm.btree {
                            Err(Error::NotSupportMask("BTree".to_string()).into())
                        } else {
                            Err(Error::NotSupportMask("Sidewinder".to_string()).into())
                        }
                    }
                }
            }
        },
        other_algorithm => unreachable!(
            "Given unknown algorithm or missing arguments of algorithm({:?}), should be refused by clap.",
            other_algorithm
        ),
    }
}
