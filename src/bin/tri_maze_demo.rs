use anyhow::Error as AnyError;
use clap::{Args, Parser, Subcommand, arg};
use try_mazes::{
    cli::{self, GeneralMazeAction, GeneralMazeLoadArgs, GeneralRectMazeShape},
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
    let maze = match &maze_input.action {
        DemoAction::Create(TriMazeCreateArgs { algorithm, shape }) => {
            let grid = match shape {
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
            let generator: &dyn TriMazeGenerator = match algorithm {
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
            generator.generate(grid)
        }
        DemoAction::Load(GeneralMazeLoadArgs { load_path, .. }) => cli::load_from_json(load_path)?,
    };

    let painter = TriMazePainter::new(&maze, maze_input.cell_height, maze_input.wall_thickness);
    let picture = MazePicture::new(&painter);
    match &maze_input.action {
        DemoAction::Create(TriMazeCreateArgs {
            shape: GeneralRectMazeShape::Size { action, .. },
            ..
        })
        | DemoAction::Create(TriMazeCreateArgs {
            shape: GeneralRectMazeShape::Mask { action, .. },
            ..
        })
        | DemoAction::Load(GeneralMazeLoadArgs { action, .. }) => match action {
            GeneralMazeAction::Show {
                wnd_width,
                wnd_height,
            } => picture.show(*wnd_width, *wnd_height)?,
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
struct TriMazeInputArgs {
    /// Height of cell space
    #[arg(short, long, default_value_t = DEF_TRI_CELL_HEIGHT)]
    cell_height: u16,
    /// Thickness of the maze wall(the stroke)
    #[arg(short, long, default_value_t = DEF_WALL_THICKNESS)]
    wall_thickness: u16,
    /// What to do in demo
    #[command(subcommand)]
    action: DemoAction,
}

#[derive(Debug, Clone, Subcommand)]
enum DemoAction {
    Create(TriMazeCreateArgs),
    Load(GeneralMazeLoadArgs),
}

#[derive(Debug, Clone, Args)]
struct TriMazeCreateArgs {
    /// Maze generation algorithm
    #[command(flatten)]
    algorithm: TriMazeAlgorithm,
    /// Specified maze shape
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
