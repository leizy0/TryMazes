use try_mazes::gene::BTreeMazeGenerator;

fn main() {
    let generator = BTreeMazeGenerator::new();
    let maze = generator.generate(5, 4);
    println!("{}", maze);
}
