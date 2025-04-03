use try_mazes::gene::BTreeMazeGenerator;

fn main() {
    let generator = BTreeMazeGenerator::new();
    let maze = generator.generate(15, 8);
    println!("{}", maze);
}
