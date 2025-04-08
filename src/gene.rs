use clap::ValueEnum;
use rand::Rng;

use crate::maze::{Direction, Maze};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Hash)]
pub enum DiagonalDirection {
    Northeast,
    Southeast,
    Southwest,
    Northwest,
}

impl DiagonalDirection {
    pub fn hv_dirs(&self) -> (Direction, Direction) {
        match self {
            DiagonalDirection::Northeast => (Direction::East, Direction::North),
            DiagonalDirection::Southeast => (Direction::East, Direction::South),
            DiagonalDirection::Southwest => (Direction::West, Direction::South),
            DiagonalDirection::Northwest => (Direction::West, Direction::North),
        }
    }
}

pub trait MazeGenerator {
    fn generate(&self, width: usize, height: usize) -> Maze;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BTreeMazeGenerator {
    con_dir: DiagonalDirection,
}

impl BTreeMazeGenerator {
    pub fn new(con_dir: DiagonalDirection) -> Self {
        Self { con_dir }
    }
}

impl MazeGenerator for BTreeMazeGenerator {
    fn generate(&self, width: usize, height: usize) -> Maze {
        let mut maze = Maze::new(width, height);
        let mut rng = rand::rng();
        let (horz_dir, vert_dir) = self.con_dir.hv_dirs();
        let connect_dirs = [horz_dir, vert_dir];
        for r_ind in 0..height {
            for c_ind in 0..width {
                let at_horz_border = maze.is_at_border(r_ind, c_ind, horz_dir);
                let at_vert_border = maze.is_at_border(r_ind, c_ind, vert_dir);

                if at_horz_border {
                    if !at_vert_border {
                        maze.connect_to(r_ind, c_ind, vert_dir);
                    }
                } else if at_vert_border {
                    maze.connect_to(r_ind, c_ind, horz_dir);
                } else {
                    let rand_dir = connect_dirs[rng.random_range(0..connect_dirs.len())];
                    maze.connect_to(r_ind, c_ind, rand_dir);
                }
            }
        }

        maze
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SideWinderMazeGenerator {
    con_dir: DiagonalDirection,
}

impl SideWinderMazeGenerator {
    pub fn new(con_dir: DiagonalDirection) -> Self {
        Self { con_dir }
    }
}

impl MazeGenerator for SideWinderMazeGenerator {
    fn generate(&self, width: usize, height: usize) -> Maze {
        let mut maze = Maze::new(width, height);
        let mut rng = rand::rng();
        let (horz_dir, vert_dir) = self.con_dir.hv_dirs();
        let is_horz_reverse = horz_dir == Direction::West;
        for r_ind in 0..height {
            let mut run_start_ind = if is_horz_reverse { width - 1 } else { 0 };
            for c_ind in 0..width {
                let c_ind = if is_horz_reverse {
                    width - 1 - c_ind
                } else {
                    c_ind
                };
                let at_horz_border = maze.is_at_border(r_ind, c_ind, horz_dir);
                let at_vert_border = maze.is_at_border(r_ind, c_ind, vert_dir);
                let close_out = !at_vert_border && (at_horz_border || rng.random::<bool>());

                if close_out {
                    let out_ind = if is_horz_reverse {
                        rng.random_range(c_ind..=run_start_ind)
                    } else {
                        rng.random_range(run_start_ind..=c_ind)
                    };
                    maze.connect_to(r_ind, out_ind, vert_dir);
                    run_start_ind = if is_horz_reverse {
                        c_ind.checked_sub(1).unwrap_or(0)
                    } else {
                        c_ind + 1
                    };
                } else if !at_horz_border {
                    maze.connect_to(r_ind, c_ind, horz_dir);
                }
            }
        }

        maze
    }
}
