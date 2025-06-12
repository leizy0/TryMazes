use std::ops::Range;

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
pub struct RectLayerMazeGenerator<G: LayerMazeGenerator> {
    generator: G,
}

impl<G: LayerMazeGenerator> RectMazeGenerator<NoMask> for RectLayerMazeGenerator<G> {
    fn generate(&self, mut grid: RectGrid<NoMask>) -> RectMaze {
        self.generator.generate_layer(&mut grid);
        RectMaze::NoMask(grid)
    }
}

impl<G: LayerMazeGenerator> RectLayerMazeGenerator<G> {
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
                        // Can only connect along the vertical direction.
                        grid.connect_to(&pos, vert_dir);
                    }
                } else if at_vert_border {
                    // Can only connect along the horizontal direction.
                    grid.connect_to(&pos, horz_dir);
                } else {
                    // Choose a direction equally likely to connect.
                    let rand_dir = connect_dirs[rng.random_range(0..connect_dirs.len())];
                    grid.connect_to(&pos, rand_dir);
                }
            }
        }
        RectMaze::NoMask(grid)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SidewinderMazeGenerator {
    con_dir: DiagonalDirection,
}

impl SidewinderMazeGenerator {
    pub fn new(con_dir: DiagonalDirection) -> Self {
        Self { con_dir }
    }
}

impl RectMazeGenerator<NoMask> for SidewinderMazeGenerator {
    fn generate(&self, mut grid: RectGrid<NoMask>) -> RectMaze {
        let (width, height) = grid.size();
        let mut rng = rand::rng();
        let (horz_dir, vert_dir) = self.con_dir.hv_dirs();
        let is_horz_reverse = horz_dir == RectDirection::West;
        for r_ind in 0..height {
            let mut run_start_ind = if is_horz_reverse { width - 1 } else { 0 };
            for c_ind in 0..width {
                let c_ind = if is_horz_reverse {
                    // Start in reverse order, from the larger one to the smaller one.
                    width - 1 - c_ind
                } else {
                    c_ind
                };
                let pos = RectPosition::new(r_ind, c_ind);
                let at_horz_border = grid.is_at_border(&pos, horz_dir);
                let at_vert_border = grid.is_at_border(&pos, vert_dir);
                let close_out = !at_vert_border && (at_horz_border || rng.random::<bool>());

                if close_out {
                    // Select a position to break out(connect to other rows) equally likely in the current run.
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
                    // if not going to connect to other rows, connect to the neighbor in the same row.
                    grid.connect_to(&pos, horz_dir);
                }
            }
        }
        RectMaze::NoMask(grid)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RecursiveDivisionMazeGenerator {
    room_max_cols_n: usize,
    room_max_rows_n: usize,
}

impl RectMazeGenerator<NoMask> for RecursiveDivisionMazeGenerator {
    fn generate(&self, mut grid: RectGrid<NoMask>) -> RectMaze {
        let (width, height) = grid.size();
        let mut rng = rand::rng();
        self.divide(&mut grid, 0..height, 0..width, &mut rng);
        RectMaze::NoMask(grid)
    }
}

impl RecursiveDivisionMazeGenerator {
    pub fn new(room_max_rows_n: usize, room_max_cols_n: usize) -> Self {
        Self {
            room_max_cols_n,
            room_max_rows_n,
        }
    }

    fn divide(
        &self,
        grid: &mut RectGrid<NoMask>,
        row_range: Range<usize>,
        col_range: Range<usize>,
        rng: &mut impl Rng,
    ) {
        let rows_n = row_range.len();
        let cols_n = col_range.len();
        if self.is_room(row_range.clone(), col_range.clone()) {
            // Break all the walls in the current room(given ranges) to make a room.
            Self::connect_all(grid, row_range.clone(), col_range.clone());
            return;
        }

        if rows_n > cols_n {
            // Divide horizontally
            let upper_last_row = row_range.start + rng.random_range(0..rows_n - 1);
            let break_col = rng.random_range(col_range.clone());
            grid.connect_to(
                &RectPosition::new(upper_last_row, break_col),
                RectDirection::South,
            );

            // Divide the two new areas recursively.
            self.divide(
                grid,
                row_range.start..upper_last_row + 1,
                col_range.clone(),
                rng,
            );
            self.divide(grid, (upper_last_row + 1)..row_range.end, col_range, rng);
        } else {
            // Divide vertically
            let left_last_col = col_range.start + rng.random_range(0..cols_n - 1);
            let break_row = rng.random_range(row_range.clone());
            grid.connect_to(
                &RectPosition::new(break_row, left_last_col),
                RectDirection::East,
            );
            
            // Divide the two new areas recursively.
            self.divide(
                grid,
                row_range.clone(),
                col_range.start..left_last_col + 1,
                rng,
            );
            self.divide(grid, row_range, (left_last_col + 1)..col_range.end, rng);
        }
    }

    fn is_room(&self, row_range: Range<usize>, col_range: Range<usize>) -> bool {
        let rows_n = row_range.len();
        let cols_n = col_range.len();
        rows_n <= 1
            || cols_n <= 1
            || (rows_n <= self.room_max_rows_n && cols_n <= self.room_max_cols_n)
    }

    fn connect_all(grid: &mut RectGrid<NoMask>, row_range: Range<usize>, col_range: Range<usize>) {
        let Some(end_row) = row_range.clone().last() else {
            return;
        };
        let Some(end_col) = col_range.clone().last() else {
            return;
        };
        for row in row_range {
            for col in col_range.clone() {
                let pos = RectPosition::new(row, col);
                if row != end_row {
                    grid.connect_to(&pos, RectDirection::South);
                }

                if col != end_col {
                    grid.connect_to(&pos, RectDirection::East);
                }
            }
        }
    }
}
