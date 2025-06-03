use std::{fmt::Display, fs::File, io::Write, path::PathBuf};

use anyhow::Error as AnyError;
use clap::{Args, Parser, Subcommand};

use try_mazes::{
    cli::Error,
    gene::{
        AldousBroderMazeGenerator, EllerMazeGenerator, GrowingTreeMazeGenerator,
        HuntAndKillMazeGenerator, KruskalMazeGenerator, PrimMazeGenerator,
        RecursiveBacktrackerMazeGenerator, WilsonMazeGenerator,
        rect::{
            BTreeMazeGenerator, DiagonalDirection, RectLayerMazeGenerator, RectMaze2dGenerator,
            RectMazeGenerator, RecursiveDivisionMazeGenerator, SideWinderMazeGenerator,
        },
    },
    maze::{
        NoMask, WithMask,
        rect::{RectGrid, RectMask},
    },
    show::{
        MazePicture, SavePictureFormat,
        rect::{AsciiBoxCharset, RectMazeCmdDisplay, RectMazePainter, UnicodeBoxCharset},
    },
};

const DEF_WALL_THICKNESS: usize = 5;
const DEF_CELL_WIDTH: usize = 50;

fn main() -> Result<(), AnyError> {
    let maze_input = RectMazeInputArgs::parse();
    let maze = match &maze_input.action {
        MazeAction::Show(ShowArgs { shape, .. }) | MazeAction::Save(SaveArgs { shape, .. }) => {
            match shape {
                MazeShape::Size(MazeSize { width, height }) => {
                    let grid = RectGrid::<NoMask>::new(*width, *height);
                    let generator = make_generator_no_mask(&maze_input);
                    generator.generate(grid)
                }
                MazeShape::Mask(mask_info) => {
                    let grid = match mask_info {
                        MazeMaskInfo {
                            text: true,
                            path: Some(mask_path),
                            ..
                        } => RectGrid::<WithMask>::new(&RectMask::try_from_text_file(mask_path)?),
                        MazeMaskInfo {
                            image: true,
                            path: Some(mask_path),
                            ..
                        } => RectGrid::<WithMask>::new(&RectMask::try_from_image_file(mask_path)?),
                        other_shape => unreachable!(
                            "Given invalid shape information({:?}), should be refused by clap.",
                            other_shape
                        ),
                    };

                    let generator = make_generator_with_mask(&maze_input)?;
                    generator.generate(grid)
                }
            }
        }
    };
    // let grid = make_grid(&maze_input)?;
    // let generator = make_generator(&maze_input)?;
    // let maze = generator.generate(grid);
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
    /// Max rows number of room, used by recursive division algorithm
    #[arg(long, default_value_t = 1)]
    room_max_rows_n: usize,
    /// Max columns number of room, used by recursive division algorithm
    #[arg(long, default_value_t = 1)]
    room_max_cols_n: usize,
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
    /// Using growing tree algorithm
    #[arg(long)]
    growing_tree: bool,
    /// Using Eller's algorithm
    #[arg(long)]
    eller: bool,
    /// Using recursive division algorithm
    #[arg(long)]
    recursive_division: bool,
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
    width: usize,
    /// Height of maze
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

fn make_generator_no_mask(input: &RectMazeInputArgs) -> Box<dyn RectMazeGenerator<NoMask>> {
    match input.algorithm {
        RectMazeGenAlgorithm {
            aldous_broder: true,
            ..
        } => Box::new(RectMaze2dGenerator::new(AldousBroderMazeGenerator)),
        RectMazeGenAlgorithm { wilson: true, .. } => {
            Box::new(RectMaze2dGenerator::new(WilsonMazeGenerator))
        }
        RectMazeGenAlgorithm {
            hunt_and_kill: true,
            ..
        } => Box::new(RectMaze2dGenerator::new(HuntAndKillMazeGenerator)),
        RectMazeGenAlgorithm {
            recursive_backtracker: true,
            ..
        } => Box::new(RectMaze2dGenerator::new(RecursiveBacktrackerMazeGenerator)),
        RectMazeGenAlgorithm { kruskal: true, .. } => {
            Box::new(RectMaze2dGenerator::new(KruskalMazeGenerator))
        }
        RectMazeGenAlgorithm { prim: true, .. } => {
            Box::new(RectMaze2dGenerator::new(PrimMazeGenerator))
        }
        RectMazeGenAlgorithm {
            growing_tree: true, ..
        } => Box::new(RectMaze2dGenerator::new(GrowingTreeMazeGenerator)),
        RectMazeGenAlgorithm { eller: true, .. } => {
            Box::new(RectLayerMazeGenerator::new(EllerMazeGenerator))
        }
        RectMazeGenAlgorithm { btree: true, .. } => {
            Box::new(BTreeMazeGenerator::new(input.con_dir.unwrap()))
        }
        RectMazeGenAlgorithm {
            sidewinder: true, ..
        } => Box::new(SideWinderMazeGenerator::new(input.con_dir.unwrap())),
        RectMazeGenAlgorithm {
            recursive_division: true,
            ..
        } => Box::new(RecursiveDivisionMazeGenerator::new(
            input.room_max_rows_n,
            input.room_max_cols_n,
        )),
        other_algorithm => unreachable!(
            "Given unknown algorithm or missing arguments of algorithm({:?}), should be refused by clap.",
            other_algorithm
        ),
    }
}

fn make_generator_with_mask(
    input: &RectMazeInputArgs,
) -> Result<Box<dyn RectMazeGenerator<WithMask>>, AnyError> {
    match input.algorithm {
        RectMazeGenAlgorithm {
            aldous_broder: true,
            ..
        } => Ok(Box::new(RectMaze2dGenerator::new(
            AldousBroderMazeGenerator,
        ))),
        RectMazeGenAlgorithm { wilson: true, .. } => {
            Ok(Box::new(RectMaze2dGenerator::new(WilsonMazeGenerator)))
        }
        RectMazeGenAlgorithm {
            hunt_and_kill: true,
            ..
        } => Ok(Box::new(RectMaze2dGenerator::new(HuntAndKillMazeGenerator))),
        RectMazeGenAlgorithm {
            recursive_backtracker: true,
            ..
        } => Ok(Box::new(RectMaze2dGenerator::new(
            RecursiveBacktrackerMazeGenerator,
        ))),
        RectMazeGenAlgorithm { kruskal: true, .. } => {
            Ok(Box::new(RectMaze2dGenerator::new(KruskalMazeGenerator)))
        }
        RectMazeGenAlgorithm { prim: true, .. } => {
            Ok(Box::new(RectMaze2dGenerator::new(PrimMazeGenerator)))
        }
        RectMazeGenAlgorithm {
            growing_tree: true, ..
        } => Ok(Box::new(RectMaze2dGenerator::new(GrowingTreeMazeGenerator))),
        RectMazeGenAlgorithm { eller: true, .. } => {
            Err(Error::NotSupportMask("Eller".to_string()).into())
        }
        RectMazeGenAlgorithm { btree: true, .. } => {
            Err(Error::NotSupportMask("BTree".to_string()).into())
        }
        RectMazeGenAlgorithm {
            sidewinder: true, ..
        } => Err(Error::NotSupportMask("Sidewinder".to_string()).into()),
        other_algorithm => unreachable!(
            "Given unknown algorithm or missing arguments of algorithm({:?}), should be refused by clap.",
            other_algorithm
        ),
    }
}
