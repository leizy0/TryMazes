use clap::ValueEnum;
use rand::Rng;

use crate::maze::rect::{RectDirection, RectGrid, RectMaze, RectPosition};

use super::Maze2dGenerator;

pub trait RectMazeGenerator {
    fn generate(&self, grid: RectGrid) -> RectMaze;
}

impl<T: Maze2dGenerator> RectMazeGenerator for T {
    fn generate(&self, mut grid: RectGrid) -> RectMaze {
        self.generate_2d(&mut grid);
        RectMaze::new(grid)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Hash)]
pub enum DiagonalDirection {
    Northeast,
    Southeast,
    Southwest,
    Northwest,
}

impl DiagonalDirection {
    pub fn hv_dirs(&self) -> (RectDirection, RectDirection) {
        match self {
            DiagonalDirection::Northeast => (RectDirection::East, RectDirection::North),
            DiagonalDirection::Southeast => (RectDirection::East, RectDirection::South),
            DiagonalDirection::Southwest => (RectDirection::West, RectDirection::South),
            DiagonalDirection::Northwest => (RectDirection::West, RectDirection::North),
        }
    }
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

impl RectMazeGenerator for BTreeMazeGenerator {
    fn generate(&self, mut grid: RectGrid) -> RectMaze {
        let (width, height) = grid.size();
        let mut rng = rand::rng();
        let (horz_dir, vert_dir) = self.con_dir.hv_dirs();
        let connect_dirs = [horz_dir, vert_dir];
        for r_ind in 0..height {
            for c_ind in 0..width {
                let pos = RectPosition::new(r_ind, c_ind);
                let at_horz_border = grid.is_at_border(&pos, horz_dir);
                let at_vert_border = grid.is_at_border(&pos, vert_dir);

                if at_horz_border {
                    if !at_vert_border {
                        grid.connect_along(&pos, vert_dir);
                    }
                } else if at_vert_border {
                    grid.connect_along(&pos, horz_dir);
                } else {
                    let rand_dir = connect_dirs[rng.random_range(0..connect_dirs.len())];
                    grid.connect_along(&pos, rand_dir);
                }
            }
        }

        RectMaze::new(grid)
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

impl RectMazeGenerator for SideWinderMazeGenerator {
    fn generate(&self, mut grid: RectGrid) -> RectMaze {
        let (width, height) = grid.size();
        let mut rng = rand::rng();
        let (horz_dir, vert_dir) = self.con_dir.hv_dirs();
        let is_horz_reverse = horz_dir == RectDirection::West;
        for r_ind in 0..height {
            let mut run_start_ind = if is_horz_reverse { width - 1 } else { 0 };
            for c_ind in 0..width {
                let c_ind = if is_horz_reverse {
                    width - 1 - c_ind
                } else {
                    c_ind
                };
                let pos = RectPosition::new(r_ind, c_ind);
                let at_horz_border = grid.is_at_border(&pos, horz_dir);
                let at_vert_border = grid.is_at_border(&pos, vert_dir);
                let close_out = !at_vert_border && (at_horz_border || rng.random::<bool>());

                if close_out {
                    let out_ind = if is_horz_reverse {
                        rng.random_range(c_ind..=run_start_ind)
                    } else {
                        rng.random_range(run_start_ind..=c_ind)
                    };
                    grid.connect_along(&RectPosition::new(r_ind, out_ind), vert_dir);
                    run_start_ind = if is_horz_reverse {
                        c_ind.saturating_sub(1)
                    } else {
                        c_ind + 1
                    };
                } else if !at_horz_border {
                    grid.connect_along(&pos, horz_dir);
                }
            }
        }

        RectMaze::new(grid)
    }
}
