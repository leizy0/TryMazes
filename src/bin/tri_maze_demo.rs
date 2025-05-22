use anyhow::Error as AnyError;
use clap::Parser;
use try_mazes::{
    cli::{GeneralMaze2dAlgorithm, GeneralRectMazeShape, MazeAction},
    gene::{
        AldousBroderMazeGenerator, GrowingTreeMazeGenerator, HuntAndKillMazeGenerator,
        KruskalMazeGenerator, PrimMazeGenerator, RecursiveBacktrackerMazeGenerator,
        WilsonMazeGenerator, tri::TriMazeGenerator,
    },
    maze::{rect::RectMask, tri::TriGrid},
    show::{MazePicture, tri::TriMazePainter},
};

const DEF_TRI_CELL_HEIGHT: u16 = 50;
const DEF_WALL_THICKNESS: u16 = 5;

fn main() -> Result<(), AnyError> {
    let maze_input = HexaMazeInputArgs::parse();
    let grid = match &maze_input.shape {
        GeneralRectMazeShape::Size { width, height, .. } => TriGrid::new(*width, *height),
        GeneralRectMazeShape::Mask {
            text: true, path, ..
        } => TriGrid::with_mask(&RectMask::try_from_text_file(path)?),
        GeneralRectMazeShape::Mask {
            image: true, path, ..
        } => TriGrid::with_mask(&RectMask::try_from_image_file(path)?),
        other_shape => unreachable!(
            "Invalid maze shape({:?}), should be refused by clap.",
            other_shape
        ),
    };
    let generator: &dyn TriMazeGenerator = match maze_input.algorithm {
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
        GeneralMaze2dAlgorithm { kruskal: true, .. } => &KruskalMazeGenerator,
        GeneralMaze2dAlgorithm { prim: true, .. } => &PrimMazeGenerator,
        GeneralMaze2dAlgorithm {
            growing_tree: true, ..
        } => &GrowingTreeMazeGenerator,
        other_algorithm => unreachable!(
            "Invalid algorithm({:?}), should be refused by clap.",
            other_algorithm
        ),
    };
    let maze = generator.generate(grid);
    let painter = TriMazePainter::new(&maze, maze_input.cell_height, maze_input.wall_thickness);
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
    #[arg(short, long, default_value_t = DEF_TRI_CELL_HEIGHT)]
    cell_height: u16,
    /// Thickness of the maze wall(the stroke)
    #[arg(short, long, default_value_t = DEF_WALL_THICKNESS)]
    wall_thickness: u16,
    /// Maze shape, by size or from mask
    #[command(subcommand)]
    shape: GeneralRectMazeShape,
}
