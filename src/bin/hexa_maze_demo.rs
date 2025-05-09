use std::path::PathBuf;

use anyhow::Error as AnyError;
use clap::{Parser, Subcommand};
use try_mazes::{
    cli::{GeneralMaze2dAlgorithm, MazeAction},
    gene::{
        AldousBroderMazeGenerator, HuntAndKillMazeGenerator, RecursiveBacktrackerMazeGenerator,
        WilsonMazeGenerator, hexa::HexaMazeGenerator,
    },
    maze::{hexa::HexaGrid, rect::RectMask},
    show::{MazePicture, hexa::HexaMazePainter},
};

const DEF_CELL_WIDTH: u16 = 50;
const DEF_WALL_THICKNESS: u16 = 5;

fn main() -> Result<(), AnyError> {
    let maze_input = HexaMazeInputArgs::parse();
    let grid = match &maze_input.shape {
        GeneralRectMazeShape::Size { width, height, .. } => HexaGrid::new(*width, *height),
        GeneralRectMazeShape::Mask {
            text: true, path, ..
        } => HexaGrid::with_mask(&RectMask::try_from_text_file(path)?),
        GeneralRectMazeShape::Mask {
            image: true, path, ..
        } => HexaGrid::with_mask(&RectMask::try_from_image_file(path)?),
        other_shape => unreachable!(
            "Invalid maze shape({:?}), should be refused by clap.",
            other_shape
        ),
    };
    let generator: &dyn HexaMazeGenerator = match maze_input.algorithm {
        GeneralMaze2dAlgorithm {
            aldous_broder: true,
            ..
        } => &AldousBroderMazeGenerator,
        GeneralMaze2dAlgorithm { wilson: true, .. } => &WilsonMazeGenerator,
        GeneralMaze2dAlgorithm {
            hunt_and_kill: true,
            ..
        } => &HuntAndKillMazeGenerator,
        GeneralMaze2dAlgorithm {
            recursive_backtracker: true,
            ..
        } => &RecursiveBacktrackerMazeGenerator,
        other_algorithm => unreachable!(
            "Invalid algorithm({:?}), should be refused by clap.",
            other_algorithm
        ),
    };
    let maze = generator.generate(grid);
    let painter = HexaMazePainter::new(&maze, maze_input.cell_height, maze_input.wall_thickness);
    let picture = MazePicture::new(&painter);
    match &maze_input.shape {
        GeneralRectMazeShape::Size { action, .. } | GeneralRectMazeShape::Mask { action, .. } => {
            match action {
                MazeAction::Show => picture.show()?,
                MazeAction::Save { path, format } => picture.save(path, *format)?,
            }
        }
    }

    Ok(())
}

#[derive(Debug, Clone, Parser)]
#[command(flatten_help = true)]
struct HexaMazeInputArgs {
    // Maze generation algorithm
    #[command(flatten)]
    algorithm: GeneralMaze2dAlgorithm,
    /// Height of cell space
    #[arg(short, long, default_value_t = DEF_CELL_WIDTH)]
    cell_height: u16,
    /// Thickness of the maze wall(the stroke)
    #[arg(short, long, default_value_t = DEF_WALL_THICKNESS)]
    wall_thickness: u16,
    /// Maze shape, by size or from mask
    #[command(subcommand)]
    shape: GeneralRectMazeShape,
}

#[derive(Debug, Clone, Subcommand)]
enum GeneralRectMazeShape {
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
