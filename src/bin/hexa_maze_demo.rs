use anyhow::Error as AnyError;
use clap::{Args, Parser};
use try_mazes::{
    cli::{GeneralRectMazeShape, MazeAction},
    gene::{
        AldousBroderMazeGenerator, EllerMazeGenerator, GrowingTreeMazeGenerator,
        HuntAndKillMazeGenerator, KruskalMazeGenerator, PrimMazeGenerator,
        RecursiveBacktrackerMazeGenerator, WilsonMazeGenerator,
        hexa::{HexaLayerMazeGenerator, HexaMaze2dGenerator, HexaMazeGenerator},
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
        HexaMazeAlgorithm {
            aldous_broder: true,
            ..
        } => &HexaMaze2dGenerator::new(AldousBroderMazeGenerator),
        HexaMazeAlgorithm { wilson: true, .. } => &HexaMaze2dGenerator::new(WilsonMazeGenerator),
        HexaMazeAlgorithm {
            hunt_and_kill: true,
            ..
        } => &HexaMaze2dGenerator::new(HuntAndKillMazeGenerator),
        HexaMazeAlgorithm {
            recursive_backtracker: true,
            ..
        } => &HexaMaze2dGenerator::new(RecursiveBacktrackerMazeGenerator),
        HexaMazeAlgorithm { kruskal: true, .. } => &HexaMaze2dGenerator::new(KruskalMazeGenerator),
        HexaMazeAlgorithm { prim: true, .. } => &HexaMaze2dGenerator::new(PrimMazeGenerator),
        HexaMazeAlgorithm {
            growing_tree: true, ..
        } => &HexaMaze2dGenerator::new(GrowingTreeMazeGenerator),
        HexaMazeAlgorithm { eller: true, .. } => &HexaLayerMazeGenerator::new(EllerMazeGenerator),
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
    algorithm: HexaMazeAlgorithm,
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

#[derive(Debug, Clone, Args)]
#[group(required = true, multiple = false)]
pub struct HexaMazeAlgorithm {
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
    /// Using Eller's algorithm
    #[arg(long)]
    pub eller: bool,
}
