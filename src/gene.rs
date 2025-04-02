use rand::Rng;

use crate::maze::{Direction, Maze};


#[derive(Debug, Default)]
pub struct BTreeMazeGenerator {}

impl BTreeMazeGenerator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn generate(&self, width: usize, height: usize) -> Maze {
        let mut maze = Maze::new(width, height);
        let mut rng = rand::rng();
        let connect_dirs = [Direction::North, Direction::East];
        for r_ind in 0..height {
            for c_ind in 0..width {
                match (r_ind, c_ind) {
                    // Top right corner.
                    (0, c_ind) if c_ind == width - 1 => (),
                    // Top row.
                    (0, c_ind) => maze.connect_to(0, c_ind, Direction::East),
                    // Right column.
                    (r_ind, c_ind) if c_ind == width - 1 => maze.connect_to(r_ind, width - 1, Direction::North),
                    (r_ind, c_ind) => {
                        let rand_dir = connect_dirs[rng.random_range(0..connect_dirs.len())];
                        maze.connect_to(r_ind, c_ind, rand_dir);
                    }
                }
            }
        }

        maze
    }
}