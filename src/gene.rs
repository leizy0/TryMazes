use clap::ValueEnum;
use rand::Rng;

use crate::maze::{Direction, Maze};

#[derive(Debug, Clone, ValueEnum)]
pub enum MazeGenAlgorithm {
    BTree,
    Sidewinder,
}

impl MazeGenAlgorithm {
    pub fn generator(&self) -> Box<dyn MazeGenerator> {
        match self {
            MazeGenAlgorithm::BTree => Box::new(BTreeMazeGenerator::new()),
            MazeGenAlgorithm::Sidewinder => Box::new(SideWinderMazeGenerator::new()),
        }
    }
}

pub trait MazeGenerator {
    fn generate(&self, width: usize, height: usize) -> Maze;
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct BTreeMazeGenerator {}

impl BTreeMazeGenerator {
    pub fn new() -> Self {
        Self::default()
    }
}

impl MazeGenerator for BTreeMazeGenerator {
    fn generate(&self, width: usize, height: usize) -> Maze {
        let mut maze = Maze::new(width, height);
        let mut rng = rand::rng();
        let connect_dirs = [Direction::North, Direction::East];
        for r_ind in 0..height {
            let at_north_border = r_ind == 0;
            for c_ind in 0..width {
                let at_east_border = c_ind == width - 1;

                if at_north_border {
                    if !at_east_border {
                        maze.connect_to(0, c_ind, Direction::East);
                    }
                } else if at_east_border {
                    maze.connect_to(r_ind, c_ind, Direction::North);
                } else {
                    let rand_dir = connect_dirs[rng.random_range(0..connect_dirs.len())];
                    maze.connect_to(r_ind, c_ind, rand_dir);
                }
            }
        }

        maze
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct SideWinderMazeGenerator {}

impl SideWinderMazeGenerator {
    pub fn new() -> Self {
        Self::default()
    }
}

impl MazeGenerator for SideWinderMazeGenerator {
    fn generate(&self, width: usize, height: usize) -> Maze {
        let mut maze = Maze::new(width, height);
        let mut rng = rand::rng();
        for r_ind in 0..height {
            let mut run_start_ind = 0;
            let at_north_border = r_ind == 0;
            for c_ind in 0..width {
                let at_east_border = c_ind == width - 1;
                let close_out = !at_north_border && (at_east_border || rng.random::<bool>());

                if close_out {
                    let out_ind = rng.random_range(run_start_ind..=c_ind);
                    maze.connect_to(r_ind, out_ind, Direction::North);
                    run_start_ind = c_ind + 1;
                } else if !at_east_border {
                    maze.connect_to(r_ind, c_ind, Direction::East);
                }
            }
        }

        maze
    }
}
