use std::path::PathBuf;

use anyhow::Error as AnyError;
use clap::{Args, Parser, Subcommand};
use try_mazes::{
    gene::{
        AldousBroderMazeGenerator, HuntAndKillMazeGenerator, RecursiveBacktrackerMazeGenerator,
        WilsonMazeGenerator, circ::CircMazeGenerator,
    },
    maze::circ::CircGrid,
    show::{MazePicture, SavePictureFormat, circ::CircMazePainter},
};

const DEF_WALL_THICKNESS: usize = 5;
const DEF_RING_INTERVAL_WIDTH: usize = 50;
fn main() -> Result<(), AnyError> {
    let maze_input = CircMazeInputArgs::parse();
    let grid = CircGrid::new(maze_input.rings_n);
    let generator: &dyn CircMazeGenerator = match maze_input.algorithm {
        CircMazeAlgorithm {
            aldous_broder: true,
            ..
        } => &AldousBroderMazeGenerator,
        CircMazeAlgorithm { wilson: true, .. } => &WilsonMazeGenerator,
        CircMazeAlgorithm {
            hunt_and_kill: true,
            ..
        } => &HuntAndKillMazeGenerator,
        CircMazeAlgorithm {
            recursive_backtracker: true,
            ..
        } => &RecursiveBacktrackerMazeGenerator,
        other_algorithm => unreachable!(
            "Invalid circular maze algorithm({:?}), should be refused by clap.",
            other_algorithm
        ),
    };
    let maze = generator.generate(grid);
    let painter = CircMazePainter::new(
        &maze,
        maze_input.ring_interval_width,
        maze_input.wall_thickness,
    );
    let picture = MazePicture::new(&painter);
    match maze_input.action {
        MazeAction::Show => picture.show()?,
        MazeAction::Save { path, format } => picture.save(path, format)?,
    }

    Ok(())
}

#[derive(Debug, Clone, Parser)]
#[command(flatten_help = true)]
struct CircMazeInputArgs {
    /// Number of rings in maze
    #[arg(short, long, value_name = "RING_NUMBER")]
    rings_n: usize,
    /// Algorithm used by generator
    #[command(flatten)]
    algorithm: CircMazeAlgorithm,
    /// Width of space between two adjacent rings along the radial direction
    #[arg(short = 'i', long, default_value_t = DEF_RING_INTERVAL_WIDTH)]
    ring_interval_width: usize,
    /// Width of maze wall(stroke)
    #[arg(short, long, default_value_t = DEF_WALL_THICKNESS)]
    wall_thickness: usize,
    /// Action to do with generated maze
    #[command(subcommand)]
    action: MazeAction,
}

#[derive(Debug, Clone, Args)]
#[group(required = true, multiple = false)]
struct CircMazeAlgorithm {
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
}

#[derive(Debug, Clone, Subcommand)]
enum MazeAction {
    Show,
    Save {
        #[arg(short, long)]
        path: PathBuf,
        #[arg(short, long)]
        format: SavePictureFormat,
    },
}
