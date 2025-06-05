use anyhow::Error as AnyError;
use clap::{Args, Parser, Subcommand};
use try_mazes::{
    cli::{self, Error, GeneralMazeAction, GeneralMazeLoadArgs, GeneralRectMazeShape},
    gene::{
        AldousBroderMazeGenerator, EllerMazeGenerator, GrowingTreeMazeGenerator,
        HuntAndKillMazeGenerator, KruskalMazeGenerator, PrimMazeGenerator,
        RecursiveBacktrackerMazeGenerator, WilsonMazeGenerator,
        hexa::{HexaLayerMazeGenerator, HexaMaze2dGenerator, HexaMazeGenerator},
    },
    maze::{NoMask, WithMask, hexa::HexaGrid, rect::RectMask},
    show::{MazePicture, hexa::HexaMazePainter},
};

const DEF_CELL_WIDTH: u16 = 50;
const DEF_WALL_THICKNESS: u16 = 5;

fn main() -> Result<(), AnyError> {
    let maze_input = HexaMazeInputArgs::parse();
    let maze = match &maze_input.action {
        DemoAction::Create(HexaMazeCreateArgs {
            algorithm, shape, ..
        }) => match shape {
            GeneralRectMazeShape::Size { width, height, .. } => {
                let grid = HexaGrid::<NoMask>::new(*width, *height);
                let generator = make_generator_no_mask(algorithm);
                generator.generate(grid)
            }
            mask_shape => {
                let grid = match mask_shape {
                    GeneralRectMazeShape::Mask {
                        text: true, path, ..
                    } => HexaGrid::<WithMask>::new(&RectMask::try_from_text_file(path)?),
                    GeneralRectMazeShape::Mask {
                        image: true, path, ..
                    } => HexaGrid::<WithMask>::new(&RectMask::try_from_image_file(path)?),
                    other_shape => unreachable!(
                        "Invalid maze shape({:?}), should be refused by clap.",
                        other_shape
                    ),
                };
                let generator = make_generator_with_mask(algorithm)?;
                generator.generate(grid)
            }
        },
        DemoAction::Load(GeneralMazeLoadArgs { load_path, .. }) => cli::load_from_json(load_path)?,
    };
    let painter = HexaMazePainter::new(&maze, maze_input.cell_height, maze_input.wall_thickness);
    let picture = MazePicture::new(&painter);
    match &maze_input.action {
        DemoAction::Create(HexaMazeCreateArgs {
            shape: GeneralRectMazeShape::Size { action, .. },
            ..
        })
        | DemoAction::Create(HexaMazeCreateArgs {
            shape: GeneralRectMazeShape::Mask { action, .. },
            ..
        })
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
struct HexaMazeInputArgs {
    /// Height of cell space
    #[arg(short, long, default_value_t = DEF_CELL_WIDTH)]
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
    Create(HexaMazeCreateArgs),
    Load(GeneralMazeLoadArgs),
}

#[derive(Debug, Clone, Args)]
struct HexaMazeCreateArgs {
    // Maze generation algorithm
    #[command(flatten)]
    algorithm: HexaMazeAlgorithm,
    /// Maze shape, by size or from mask
    #[command(subcommand)]
    shape: GeneralRectMazeShape,
}

#[derive(Debug, Clone, Args)]
#[group(required = true, multiple = false)]
struct HexaMazeAlgorithm {
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

fn make_generator_no_mask(algorithm: &HexaMazeAlgorithm) -> Box<dyn HexaMazeGenerator<NoMask>> {
    match algorithm {
        HexaMazeAlgorithm {
            aldous_broder: true,
            ..
        } => Box::new(HexaMaze2dGenerator::new(AldousBroderMazeGenerator)),
        HexaMazeAlgorithm { wilson: true, .. } => {
            Box::new(HexaMaze2dGenerator::new(WilsonMazeGenerator))
        }
        HexaMazeAlgorithm {
            hunt_and_kill: true,
            ..
        } => Box::new(HexaMaze2dGenerator::new(HuntAndKillMazeGenerator)),
        HexaMazeAlgorithm {
            recursive_backtracker: true,
            ..
        } => Box::new(HexaMaze2dGenerator::new(RecursiveBacktrackerMazeGenerator)),
        HexaMazeAlgorithm { kruskal: true, .. } => {
            Box::new(HexaMaze2dGenerator::new(KruskalMazeGenerator))
        }
        HexaMazeAlgorithm { prim: true, .. } => {
            Box::new(HexaMaze2dGenerator::new(PrimMazeGenerator))
        }
        HexaMazeAlgorithm {
            growing_tree: true, ..
        } => Box::new(HexaMaze2dGenerator::new(GrowingTreeMazeGenerator)),
        HexaMazeAlgorithm { eller: true, .. } => {
            Box::new(HexaLayerMazeGenerator::new(EllerMazeGenerator))
        }
        other_algorithm => unreachable!(
            "Invalid algorithm({:?}), should be refused by clap.",
            other_algorithm
        ),
    }
}

fn make_generator_with_mask(
    algorithm: &HexaMazeAlgorithm,
) -> Result<Box<dyn HexaMazeGenerator<WithMask>>, AnyError> {
    match algorithm {
        HexaMazeAlgorithm {
            aldous_broder: true,
            ..
        } => Ok(Box::new(HexaMaze2dGenerator::new(
            AldousBroderMazeGenerator,
        ))),
        HexaMazeAlgorithm { wilson: true, .. } => {
            Ok(Box::new(HexaMaze2dGenerator::new(WilsonMazeGenerator)))
        }
        HexaMazeAlgorithm {
            hunt_and_kill: true,
            ..
        } => Ok(Box::new(HexaMaze2dGenerator::new(HuntAndKillMazeGenerator))),
        HexaMazeAlgorithm {
            recursive_backtracker: true,
            ..
        } => Ok(Box::new(HexaMaze2dGenerator::new(
            RecursiveBacktrackerMazeGenerator,
        ))),
        HexaMazeAlgorithm { kruskal: true, .. } => {
            Ok(Box::new(HexaMaze2dGenerator::new(KruskalMazeGenerator)))
        }
        HexaMazeAlgorithm { prim: true, .. } => {
            Ok(Box::new(HexaMaze2dGenerator::new(PrimMazeGenerator)))
        }
        HexaMazeAlgorithm {
            growing_tree: true, ..
        } => Ok(Box::new(HexaMaze2dGenerator::new(GrowingTreeMazeGenerator))),
        HexaMazeAlgorithm { eller: true, .. } => {
            Err(Error::NotSupportMask("Eller".to_string()).into())
        }
        other_algorithm => unreachable!(
            "Invalid algorithm({:?}), should be refused by clap.",
            other_algorithm
        ),
    }
}
