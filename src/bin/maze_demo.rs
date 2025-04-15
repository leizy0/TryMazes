use std::{fmt::Display, fs::File, io::Write, path::PathBuf};

use anyhow::Error;
use clap::{Args, Parser, Subcommand, ValueEnum};
use try_mazes::{
    gene::{
        AldousBroderMazeGenerator, BTreeMazeGenerator, DiagonalDirection, HuntAndKillMazeGenerator,
        MazeGenerator, RecursiveBacktrackerMazeGenerator, SideWinderMazeGenerator,
        WilsonMazeGenerator,
    },
    show::{AsciiMazeDisplay, GUIMazeShow, SavePictureFormat, UnicodeDisplay},
};

fn main() -> Result<(), Error> {
    const DEF_WALL_THICKNESS: usize = 5;
    const DEF_CELL_WIDTH: usize = 50;

    let maze_input = MazeInputArgs::parse();
    let generator: &dyn MazeGenerator = match maze_input.algorithm {
        MazeGenAlgorithm {
            btree: true,
            con_dir: Some(dir),
            ..
        } => &BTreeMazeGenerator::new(dir),
        MazeGenAlgorithm {
            sidewinder: true,
            con_dir: Some(dir),
            ..
        } => &SideWinderMazeGenerator::new(dir),
        MazeGenAlgorithm {
            aldous_broder: true,
            ..
        } => &AldousBroderMazeGenerator,
        MazeGenAlgorithm {
            hunt_and_kill: true,
            ..
        } => &HuntAndKillMazeGenerator,
        MazeGenAlgorithm {
            recursive_backtracker: true,
            ..
        } => &RecursiveBacktrackerMazeGenerator,
        MazeGenAlgorithm { wilson: true, .. } => &WilsonMazeGenerator,
        _ => unreachable!(
            "Given unknown algorithm or missing arguments of algorithm, should be checked by clap."
        ),
    };
    let maze = generator.generate(maze_input.width, maze_input.height);
    match maze_input.action {
        MazeAction::Show { category } => match category {
            MazeShowCategory::Ascii => println!("{}", AsciiMazeDisplay(&maze)),
            MazeShowCategory::Unicode => println!("{}", UnicodeDisplay(&maze)),
            MazeShowCategory::Gui => {
                GUIMazeShow::new(&maze, DEF_WALL_THICKNESS, DEF_CELL_WIDTH).show()?
            }
        },
        MazeAction::Save(SaveOption {
            ascii,
            unicode,
            path,
            ..
        }) if ascii || unicode => {
            let mut file = File::create(path)?;
            let display: &dyn Display = if ascii {
                &AsciiMazeDisplay(&maze)
            } else {
                &UnicodeDisplay(&maze)
            };
            file.write_all(display.to_string().as_bytes())?;
            file.flush()?;
        }
        MazeAction::Save(SaveOption {
            picture: true,
            pic_format: Some(format),
            path,
            ..
        }) => {
            let mut file = File::create(path)?;
            file.write_all(
                GUIMazeShow::new(&maze, DEF_WALL_THICKNESS, DEF_CELL_WIDTH)
                    .image_data(format)?
                    .as_bytes(),
            )?;
            file.flush()?;
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
struct MazeInputArgs {
    /// Generation algorithm
    #[command(flatten)]
    algorithm: MazeGenAlgorithm,
    /// Width of maze
    width: usize,
    /// Height of maze
    height: usize,
    /// What to do with generated maze
    #[command(subcommand)]
    action: MazeAction,
}

#[derive(Debug, Clone, Copy, Args)]
struct MazeGenAlgorithm {
    /// Using binary tree algorithm
    #[arg(long, group = "algorithm", requires = "connect direction")]
    btree: bool,
    /// Using sidewinder algorithm
    #[arg(long, group = "algorithm", requires = "connect direction")]
    sidewinder: bool,
    /// Using Aldous-Broder algorithm
    #[arg(long, group = "algorithm")]
    aldous_broder: bool,
    /// Using Wilson's algorithm
    #[arg(long, group = "algorithm")]
    wilson: bool,
    /// Using Hunt-and-Kill algorithm
    #[arg(long, group = "algorithm")]
    hunt_and_kill: bool,
    /// Using recursive backtracker algorithm
    #[arg(long, group = "algorithm")]
    recursive_backtracker: bool,
    /// Candidate directions to connect
    #[arg(short, long, group = "connect direction")]
    con_dir: Option<DiagonalDirection>,
}

#[derive(Debug, Subcommand)]
enum MazeAction {
    /// Show maze by chosen way
    Show { category: MazeShowCategory },
    /// Save maze by given settings
    Save(SaveOption),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, ValueEnum)]
enum MazeShowCategory {
    /// Using ascii characters to display maze
    Ascii,
    /// Using unicode box characters to display maze
    Unicode,
    /// Using GUI to display maze in graphics
    Gui,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Args)]
struct SaveOption {
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
    #[arg(short, long)]
    path: PathBuf,
}
