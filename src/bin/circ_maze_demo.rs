use anyhow::Error as AnyError;
use clap::{Args, Parser, Subcommand};
use try_mazes::{
    cli::{self, GeneralMazeAction, GeneralMazeLoadArgs},
    gene::{
        AldousBroderMazeGenerator, EllerMazeGenerator, GrowingTreeMazeGenerator,
        HuntAndKillMazeGenerator, KruskalMazeGenerator, PrimMazeGenerator,
        RecursiveBacktrackerMazeGenerator, WilsonMazeGenerator,
        circ::{CircLayerMazeGenerator, CircMaze2dGenerator, CircMazeGenerator},
    },
    maze::circ::CircGrid,
    show::{MazePicture, circ::CircMazePainter},
};

const DEF_WALL_THICKNESS: usize = 5;
const DEF_RING_INTERVAL_WIDTH: usize = 50;
fn main() -> Result<(), AnyError> {
    let maze_input = CircMazeInputArgs::parse();
    let maze = match &maze_input.action {
        DemoAction::Create(CircMazeCreateArgs {
            rings_n, algorithm, ..
        }) => {
            let grid = CircGrid::new(*rings_n);
            let generator: &dyn CircMazeGenerator = match algorithm {
                CircMazeAlgorithm {
                    aldous_broder: true,
                    ..
                } => &CircMaze2dGenerator::new(AldousBroderMazeGenerator),
                CircMazeAlgorithm { wilson: true, .. } => {
                    &CircMaze2dGenerator::new(WilsonMazeGenerator)
                }
                CircMazeAlgorithm {
                    hunt_and_kill: true,
                    ..
                } => &CircMaze2dGenerator::new(HuntAndKillMazeGenerator),
                CircMazeAlgorithm {
                    recursive_backtracker: true,
                    ..
                } => &CircMaze2dGenerator::new(RecursiveBacktrackerMazeGenerator),
                CircMazeAlgorithm { kruskal: true, .. } => {
                    &CircMaze2dGenerator::new(KruskalMazeGenerator)
                }
                CircMazeAlgorithm { prim: true, .. } => {
                    &CircMaze2dGenerator::new(PrimMazeGenerator)
                }
                CircMazeAlgorithm {
                    growing_tree: true, ..
                } => &CircMaze2dGenerator::new(GrowingTreeMazeGenerator),
                CircMazeAlgorithm { eller: true, .. } => {
                    &CircLayerMazeGenerator::new(EllerMazeGenerator)
                }
                other_algorithm => unreachable!(
                    "Invalid circular maze algorithm({:?}), should be refused by clap.",
                    other_algorithm
                ),
            };
            generator.generate(grid)
        }
        DemoAction::Load(GeneralMazeLoadArgs { load_path, .. }) => cli::load_from_json(load_path)?,
    };

    let painter = CircMazePainter::new(
        &maze,
        maze_input.ring_interval_width,
        maze_input.wall_thickness,
    );
    let picture = MazePicture::new(&painter);
    match &maze_input.action {
        DemoAction::Create(CircMazeCreateArgs { action, .. })
        | DemoAction::Load(GeneralMazeLoadArgs { action, .. }) => match action {
            GeneralMazeAction::Show => picture.show()?,
            GeneralMazeAction::Save {
                picture: true,
                path,
                format: Some(pic_format),
                ..
            } => picture.save(path, *pic_format)?,
            GeneralMazeAction::Save {
                json: true, path, ..
            } => cli::save_to_json(path, &maze)?,
            other_action => unreachable!(
                "Invalid maze action({:?}), should be refused by clap.",
                other_action
            ),
        },
    }

    Ok(())
}

#[derive(Debug, Clone, Parser)]
#[command(flatten_help = true)]
struct CircMazeInputArgs {
    /// Width of space between two adjacent rings along the radial direction
    #[arg(short = 'i', long, default_value_t = DEF_RING_INTERVAL_WIDTH)]
    ring_interval_width: usize,
    /// Width of maze wall(stroke)
    #[arg(short, long, default_value_t = DEF_WALL_THICKNESS)]
    wall_thickness: usize,
    /// Action to do with generated maze
    #[command(subcommand)]
    action: DemoAction,
}

#[derive(Debug, Clone, Subcommand)]
enum DemoAction {
    Create(CircMazeCreateArgs),
    Load(GeneralMazeLoadArgs),
}

#[derive(Debug, Clone, Args)]
struct CircMazeCreateArgs {
    /// Number of rings in maze
    #[arg(short, long, value_name = "RING_NUMBER")]
    rings_n: usize,
    /// Algorithm used by generator
    #[command(flatten)]
    algorithm: CircMazeAlgorithm,
    /// What to do with circular maze
    #[command(subcommand)]
    action: GeneralMazeAction,
}

#[derive(Debug, Clone, Args)]
#[group(required = true, multiple = false)]
pub struct CircMazeAlgorithm {
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
