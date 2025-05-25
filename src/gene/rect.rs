use clap::ValueEnum;
use rand::Rng;

use crate::maze::{
    MaskType, NoMask, WithMask,
    rect::{RectDirection, RectGrid, RectMaze, RectPosition},
};

use super::{LayerMazeGenerator, Maze2dGenerator};

pub trait RectMazeGenerator<M: MaskType> {
    fn generate(&self, grid: RectGrid<M>) -> RectMaze;
}

#[derive(Debug)]
pub struct RectMaze2dGenerator<G: Maze2dGenerator> {
    generator: G,
}

impl<G: Maze2dGenerator> RectMazeGenerator<NoMask> for RectMaze2dGenerator<G> {
    fn generate(&self, mut grid: RectGrid<NoMask>) -> RectMaze {
        self.generator.generate_2d(&mut grid);
        RectMaze::NoMask(grid)
    }
}

impl<G: Maze2dGenerator> RectMazeGenerator<WithMask> for RectMaze2dGenerator<G> {
    fn generate(&self, mut grid: RectGrid<WithMask>) -> RectMaze {
        self.generator.generate_2d(&mut grid);
        RectMaze::WithMask(grid)
    }
}

impl<G: Maze2dGenerator> RectMaze2dGenerator<G> {
    pub fn new(generator: G) -> Self {
        Self { generator }
    }
}

#[derive(Debug)]
pub struct RectLayzerMazeGenerator<G: LayerMazeGenerator> {
    generator: G,
}

impl<G: LayerMazeGenerator> RectMazeGenerator<NoMask> for RectLayzerMazeGenerator<G> {
    fn generate(&self, mut grid: RectGrid<NoMask>) -> RectMaze {
        self.generator.generate_layer(&mut grid);
        RectMaze::NoMask(grid)
    }
}

impl<G: LayerMazeGenerator> RectLayzerMazeGenerator<G> {
    pub fn new(generator: G) -> Self {
        Self { generator }
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

impl RectMazeGenerator<NoMask> for BTreeMazeGenerator {
    fn generate(&self, mut grid: RectGrid<NoMask>) -> RectMaze {
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
                        grid.connect_to(&pos, vert_dir);
                    }
                } else if at_vert_border {
                    grid.connect_to(&pos, horz_dir);
                } else {
                    let rand_dir = connect_dirs[rng.random_range(0..connect_dirs.len())];
                    grid.connect_to(&pos, rand_dir);
                }
            }
        }
        RectMaze::NoMask(grid)
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

impl RectMazeGenerator<NoMask> for SideWinderMazeGenerator {
    fn generate(&self, mut grid: RectGrid<NoMask>) -> RectMaze {
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
                    grid.connect_to(&RectPosition::new(r_ind, out_ind), vert_dir);
                    run_start_ind = if is_horz_reverse {
                        c_ind.saturating_sub(1)
                    } else {
                        c_ind + 1
                    };
                } else if !at_horz_border {
                    grid.connect_to(&pos, horz_dir);
                }
            }
        }
        RectMaze::NoMask(grid)
    }
}
