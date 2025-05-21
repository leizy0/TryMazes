use std::{fmt::Display, fs::File, io::Write, path::PathBuf};

use anyhow::Error as AnyError;
use clap::{Args, Parser, Subcommand};
use thiserror::Error;
use try_mazes::{
    gene::{
        AldousBroderMazeGenerator, HuntAndKillMazeGenerator, KruskalMazeGenerator,
        PrimMazeGenerator, RecursiveBacktrackerMazeGenerator, WilsonMazeGenerator,
        rect::{BTreeMazeGenerator, DiagonalDirection, RectMazeGenerator, SideWinderMazeGenerator},
    },
    maze::rect::{RectGrid, RectMask},
    show::{
        MazePicture, SavePictureFormat,
        rect::{AsciiBoxCharset, RectMazeCmdDisplay, RectMazePainter, UnicodeBoxCharset},
    },
};

const DEF_WALL_THICKNESS: usize = 5;
const DEF_CELL_WIDTH: usize = 50;

fn main() -> Result<(), AnyError> {
    let maze_input = RectMazeInputArgs::parse();
    let grid = make_grid(&maze_input)?;
    let generator = make_generator(&maze_input)?;
    let maze = generator.generate(grid);
    match maze_input.action {
        MazeAction::Show(ShowArgs { ascii: true, .. }) => {
            println!("{}", RectMazeCmdDisplay(&maze, AsciiBoxCharset))
        }
        MazeAction::Show(ShowArgs { unicode: true, .. }) => {
            println!("{}", RectMazeCmdDisplay(&maze, UnicodeBoxCharset))
        }
        MazeAction::Show(ShowArgs {
            gui: true,
            pic_settings,
            ..
        }) => {
            let painter =
                RectMazePainter::new(&maze, pic_settings.wall_thickness, pic_settings.cell_width);
            let picture = MazePicture::new(&painter);
            picture.show()?
        }
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
            pic_settings,
            path,
            ..
        }) => {
            let painter =
                RectMazePainter::new(&maze, pic_settings.wall_thickness, pic_settings.cell_width);
            let picture = MazePicture::new(&painter);
            picture.save(path, format)?
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
#[command(flatten_help = true)]
struct RectMazeInputArgs {
    /// Generation algorithm
    #[command(flatten)]
    algorithm: RectMazeGenAlgorithm,
    /// Candidate directions to connect
    #[arg(short, long, group = "connect direction")]
    con_dir: Option<DiagonalDirection>,
    /// What to do with generated maze
    #[command(subcommand)]
    action: MazeAction,
}

#[derive(Debug, Clone, Copy, Args)]
#[group(required = true, multiple = false)]
struct RectMazeGenAlgorithm {
    /// Using binary tree algorithm
    #[arg(long, requires = "connect direction")]
    btree: bool,
    /// Using sidewinder algorithm
    #[arg(long, requires = "connect direction")]
    sidewinder: bool,
    /// Using Aldous-Broder algorithm
    #[arg(long)]
    aldous_broder: bool,
    /// Using Wilson's algorithm
    #[arg(long)]
    wilson: bool,
    /// Using Hunt-and-Kill algorithm
    #[arg(long)]
    hunt_and_kill: bool,
    /// Using recursive backtracker algorithm
    #[arg(long)]
    recursive_backtracker: bool,
    /// Using Kruskal's algorithm
    #[arg(long)]
    kruskal: bool,
    /// Using Prim's algorithm
    #[arg(long)]
    prim: bool,
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
    /// Settings to paint maze picture
    #[command(flatten)]
    pic_settings: PictureSettings,
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
    /// Settings to paint maze picture
    #[command(flatten)]
    pic_settings: PictureSettings,
    /// Maze shape
    #[command(subcommand)]
    shape: MazeShape,
}

#[derive(Debug, Clone, Args)]
struct PictureSettings {
    /// Width of each cell empty space
    #[arg(long, default_value_t = DEF_CELL_WIDTH)]
    cell_width: usize,
    /// Thickness of maze wall(the stroke)
    #[arg(long, default_value_t = DEF_WALL_THICKNESS)]
    wall_thickness: usize,
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

fn make_grid(input: &RectMazeInputArgs) -> Result<RectGrid, AnyError> {
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

fn make_generator(input: &RectMazeInputArgs) -> Result<Box<dyn RectMazeGenerator>, AnyError> {
    match input.algorithm {
        RectMazeGenAlgorithm {
            aldous_broder: true,
            ..
        } => Ok(Box::new(AldousBroderMazeGenerator)),
        RectMazeGenAlgorithm { wilson: true, .. } => Ok(Box::new(WilsonMazeGenerator)),
        RectMazeGenAlgorithm {
            hunt_and_kill: true,
            ..
        } => Ok(Box::new(HuntAndKillMazeGenerator)),
        RectMazeGenAlgorithm {
            recursive_backtracker: true,
            ..
        } => Ok(Box::new(RecursiveBacktrackerMazeGenerator)),
        RectMazeGenAlgorithm { kruskal: true, .. } => Ok(Box::new(KruskalMazeGenerator)),
        RectMazeGenAlgorithm { prim: true, .. } => Ok(Box::new(PrimMazeGenerator)),
        RectMazeGenAlgorithm { btree: true, .. }
        | RectMazeGenAlgorithm {
            sidewinder: true, ..
        } => match &input.action {
            MazeAction::Show(ShowArgs { shape, .. }) | MazeAction::Save(SaveArgs { shape, .. }) => {
                match shape {
                    MazeShape::Size(_) => {
                        if input.algorithm.btree {
                            Ok(Box::new(BTreeMazeGenerator::new(input.con_dir.unwrap())))
                        } else {
                            Ok(Box::new(SideWinderMazeGenerator::new(
                                input.con_dir.unwrap(),
                            )))
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
