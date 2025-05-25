use anyhow::Error as AnyError;
use clap::{Args, Parser, arg};
use try_mazes::{
    cli::{GeneralRectMazeShape, MazeAction},
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
    let maze_input = TriMazeInputArgs::parse();
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
        TriMazeAlgorithm {
            aldous_broder: true,
            ..
        } => &AldousBroderMazeGenerator,
        TriMazeAlgorithm { wilson: true, .. } => &WilsonMazeGenerator,
        TriMazeAlgorithm {
            hunt_and_kill: true,
            ..
        } => &HuntAndKillMazeGenerator,
        TriMazeAlgorithm {
            recursive_backtracker: true,
            ..
        } => &RecursiveBacktrackerMazeGenerator,
        TriMazeAlgorithm { kruskal: true, .. } => &KruskalMazeGenerator,
        TriMazeAlgorithm { prim: true, .. } => &PrimMazeGenerator,
        TriMazeAlgorithm {
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
struct TriMazeInputArgs {
    // Maze generation algorithm
    #[command(flatten)]
    algorithm: TriMazeAlgorithm,
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

#[derive(Debug, Clone, Args)]
#[group(required = true, multiple = false)]
pub struct TriMazeAlgorithm {
    /// Using Aldous-Broder algorithm
    #[arg(long)]
    pub aldous_broder: bool,
    /// Using Wilson's algorithm
    #[arg(long)]
    pub wilson: bool,
    /// Using Hunt-and-Kill algorithm
    #[arg(long)]
    pub hunt_and_kill: bool,
    /// Using recursive backtracker algorithm
    #[arg(long)]
    pub recursive_backtracker: bool,
    /// Using Kruskal's algorithm
    #[arg(long)]
    pub kruskal: bool,
    /// Using Prim's algorithm
    #[arg(long)]
    pub prim: bool,
    /// Using growing tree algorithm
    #[arg(long)]
    pub growing_tree: bool,
}
