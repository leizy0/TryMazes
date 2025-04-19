use std::{fmt::Display, fs::File, io::Write, path::PathBuf};

use anyhow::Error as AnyError;
use clap::{Args, Parser, Subcommand};
use thiserror::Error;
use try_mazes::{
    gene::{
        AldousBroderMazeGenerator, BTreeMazeGenerator, DiagonalDirection, HuntAndKillMazeGenerator,
        MazeGenerator, RecursiveBacktrackerMazeGenerator, SideWinderMazeGenerator,
        WilsonMazeGenerator,
    },
    maze::Mask,
    show::{AsciiMazeDisplay, GUIMazeShow, SavePictureFormat, UnicodeDisplay},
};

fn main() -> Result<(), AnyError> {
    const DEF_WALL_THICKNESS: usize = 5;
    const DEF_CELL_WIDTH: usize = 50;

    let maze_input = MazeInputArgs::parse();
    let generator = make_generator(&maze_input)?;
    let maze = generator.generate();
    match maze_input.action {
        MazeAction::Show(ShowArgs { ascii: true, .. }) => println!("{}", AsciiMazeDisplay(&maze)),
        MazeAction::Show(ShowArgs { unicode: true, .. }) => println!("{}", UnicodeDisplay(&maze)),
        MazeAction::Show(ShowArgs { gui: true, .. }) => {
            GUIMazeShow::new(&maze, DEF_WALL_THICKNESS, DEF_CELL_WIDTH).show()?
        }
        MazeAction::Save(SaveArgs {
            ascii,
            unicode,
            path,
            ..
        }) if ascii || unicode => {
            let mut file = File::create(path)?;
            let display: &dyn Display = if ascii {
                &AsciiMazeDisplay(&maze)
            } else {
                &UnicodeDisplay(&maze)
            };
            file.write_all(display.to_string().as_bytes())?;
            file.flush()?;
        }
        MazeAction::Save(SaveArgs {
            picture: true,
            pic_format: Some(format),
            path,
            ..
        }) => {
            let mut file = File::create(path)?;
            file.write_all(
                GUIMazeShow::new(&maze, DEF_WALL_THICKNESS, DEF_CELL_WIDTH)
                    .image_data(format)?
                    .as_bytes(),
            )?;
            file.flush()?;
        }
        _ => unreachable!(
            "Given unknown action or missing arguments of action, should be checked by clap."
        ),
    }

    Ok(())
}

#[derive(Debug, Parser)]
#[command(name = "MazeDemo", version)]
#[command(about = "Demo of maze generation and display(on command line).", long_about = None)]
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
    #[arg(short, long)]
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
    #[arg(long, group = "mask info")]
    path: Option<PathBuf>,
}

#[derive(Debug, Clone, Error)]
enum Error {
    #[error("Algorithm {0} doesn't support mask.")]
    NotSupportMask(String),
}

fn make_generator(input: &MazeInputArgs) -> Result<Box<dyn MazeGenerator>, AnyError> {
    match &input.action {
        MazeAction::Show(ShowArgs { shape, .. }) | MazeAction::Save(SaveArgs { shape, .. }) => {
            match shape {
                MazeShape::Size(MazeSize { width, height }) => {
                    let width = *width;
                    let height = *height;
                    Ok(match input.algorithm {
                        MazeGenAlgorithm {
                            btree: true,
                            con_dir: Some(dir),
                            ..
                        } => Box::new(BTreeMazeGenerator::new(width, height, dir)),
                        MazeGenAlgorithm {
                            sidewinder: true,
                            con_dir: Some(dir),
                            ..
                        } => Box::new(SideWinderMazeGenerator::new(width, height, dir)),
                        MazeGenAlgorithm {
                            aldous_broder: true,
                            ..
                        } => Box::new(AldousBroderMazeGenerator::new(width, height)),
                        MazeGenAlgorithm {
                            hunt_and_kill: true,
                            ..
                        } => Box::new(HuntAndKillMazeGenerator::new(width, height)),
                        MazeGenAlgorithm {
                            recursive_backtracker: true,
                            ..
                        } => Box::new(RecursiveBacktrackerMazeGenerator::new(width, height)),
                        MazeGenAlgorithm { wilson: true, .. } => {
                            Box::new(WilsonMazeGenerator::new(width, height))
                        }
                        _ => unreachable!(
                            "Given unknown algorithm or missing arguments of algorithm, should be checked by clap."
                        ),
                    })
                }
                MazeShape::Mask(mask_info) => {
                    let mask = match mask_info {
                        MazeMaskInfo {
                            text: true,
                            path: Some(mask_path),
                            ..
                        } => Mask::try_from_text_file(mask_path)?,
                        MazeMaskInfo {
                            image: true,
                            path: Some(mask_path),
                            ..
                        } => Mask::try_from_image_file(mask_path)?,
                        other_info => unreachable!(
                            "Given invalid mask information({:?}), should be refused by clap.",
                            other_info
                        ),
                    };
                    match input.algorithm {
                        MazeGenAlgorithm { btree: true, .. } => {
                            Err(Error::NotSupportMask("BTree".to_string()).into())
                        }
                        MazeGenAlgorithm {
                            sidewinder: true, ..
                        } => Err(Error::NotSupportMask("Sidewinder".to_string()).into()),
                        MazeGenAlgorithm {
                            aldous_broder: true,
                            ..
                        } => Ok(Box::new(AldousBroderMazeGenerator::with_mask(mask))),
                        MazeGenAlgorithm {
                            hunt_and_kill: true,
                            ..
                        } => Ok(Box::new(HuntAndKillMazeGenerator::with_mask(mask))),
                        MazeGenAlgorithm {
                            recursive_backtracker: true,
                            ..
                        } => Ok(Box::new(RecursiveBacktrackerMazeGenerator::with_mask(mask))),
                        MazeGenAlgorithm { wilson: true, .. } => {
                            Ok(Box::new(WilsonMazeGenerator::with_mask(mask)))
                        }
                        other_algorithm => unreachable!(
                            "Given unknown algorithm or missing arguments of algorithm({:?}), should be refused by clap.",
                            other_algorithm
                        ),
                    }
                }
            }
        }
    }
}
