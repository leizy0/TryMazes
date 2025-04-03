use clap::Parser;
use try_mazes::gene::MazeGenAlgorithm;

fn main() {
    let maze_input = MazeInputArgs::parse();
    let generator = maze_input.algorithm.generator();
    let maze = generator.generate(maze_input.width, maze_input.height);
    println!("{}", maze);
}

#[derive(Debug, Parser)]
#[command(name = "MazeDemo", version)]
#[command(about = "Demo of maze generation and display(on command line).", long_about = None)]
struct MazeInputArgs {
    /// Generation algorithm
    #[arg(short, long)]
    algorithm: MazeGenAlgorithm,
    /// Width of maze
    width: usize,
    /// Height of maze
    height: usize,
}
