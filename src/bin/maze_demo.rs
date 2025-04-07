use std::{fs::File, io::Write, path::PathBuf};

use clap::{Args, Parser, Subcommand, ValueEnum};
use try_mazes::{gene::{BTreeMazeGenerator, ConnectDirection, MazeGenerator, SideWinderMazeGenerator}, show::AsciiMazeDisplay};

fn main() -> Result<(), std::io::Error> {
    let maze_input = MazeInputArgs::parse();
    let generator: &dyn MazeGenerator = match maze_input.algorithm {
        MazeGenAlgorithm { btree: true, con_dir: Some(dir), ..} => &BTreeMazeGenerator::new(),
        MazeGenAlgorithm { sidewinder: true, con_dir: Some(dir) , ..} => &SideWinderMazeGenerator::new(),
        _ => unreachable!("Given unknown algorithm or missing arguments of algorithm, should be checked by clap."),
    };
    let maze = generator.generate(maze_input.width, maze_input.height);
    match maze_input.action {
        MazeAction::Show { category } => {
                        match category {
                            MazeShowCategory::ASCII => println!("{}", maze),
                            MazeShowCategory::UNICODE => unimplemented!("Unicode display isn't supported yet."),
                            MazeShowCategory::GUI =>  unimplemented!("GUI display isn't supported yet."),
                        }
            },
        MazeAction::Save(SaveOption { ascii: true, path, .. }) | MazeAction::Save(SaveOption { unicode: true, path, .. }) => {
            let mut file = File::create(path)?;
            file.write(AsciiMazeDisplay(&maze).to_string().as_bytes())?;
            file.flush()?;
        },
        MazeAction::Save(SaveOption { picture: true, pic_format: Some(format), path, .. }) => unimplemented!("Saving maze to picture isn't supported yet."),
        _ => unreachable!("Given unknown action or missing arguments of action, should be checked by clap."),
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
    /// Candidate directions to connect
    #[arg(short, long, group = "connect direction")]
    con_dir: Option<ConnectDirection>,
}

#[derive(Debug)]
#[derive(Subcommand)]
enum MazeAction {
    /// Show maze by chosen way
    Show {
        category: MazeShowCategory,
    },
    /// Save maze by given settings
    Save(SaveOption),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, ValueEnum)]
enum MazeShowCategory {
    /// Using ascii characters to display maze
    ASCII,
    /// Using unicode box characters to display maze
    UNICODE,
    /// Using GUI to display maze in graphics
    GUI,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, ValueEnum)]
enum SavePictureFormat {
    /// PNG file format
    PNG,
    /// JPEG file format
    JPG,
}