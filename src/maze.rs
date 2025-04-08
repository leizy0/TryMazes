use std::fmt::Display;

use crate::show::AsciiMazeDisplay;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

#[derive(Debug, Clone, Default)]
pub struct Maze {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
}

impl Maze {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            cells: vec![Cell::default(); width * height],
        }
    }

    pub fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn is_at_border(&self, r_ind: usize, c_ind: usize, dir: Direction) -> bool {
        self.cell(r_ind, c_ind).is_some()
            && match dir {
                Direction::North => r_ind == 0,
                Direction::South => r_ind == self.height - 1,
                Direction::East => c_ind == self.width - 1,
                Direction::West => c_ind == 0,
            }
    }

    pub fn is_connect_to(&self, r_ind: usize, c_ind: usize, dir: Direction) -> bool {
        if let Some(cell) = self.cell(r_ind, c_ind) {
            match dir {
                Direction::North => cell.is_connected_to_north,
                Direction::East => cell.is_connected_to_east,
                Direction::South => self.is_connect_to(r_ind + 1, c_ind, Direction::North),
                Direction::West => c_ind
                    .checked_sub(1)
                    .map(|west_c_ind| self.is_connect_to(r_ind, west_c_ind, Direction::East))
                    .unwrap_or(false),
            }
        } else {
            false
        }
    }

    pub fn connect_to(&mut self, r_ind: usize, c_ind: usize, dir: Direction) {
        if let Some(cell) = self.cell_mut(r_ind, c_ind) {
            match dir {
                Direction::North => cell.is_connected_to_north = true,
                Direction::East => cell.is_connected_to_east = true,
                Direction::South => self.connect_to(r_ind + 1, c_ind, Direction::North),
                Direction::West => {
                    if let Some(west_c_ind) = c_ind.checked_sub(1) {
                        self.connect_to(r_ind, west_c_ind, Direction::East);
                    }
                }
            }
        }
    }

    fn cell(&self, r_ind: usize, c_ind: usize) -> Option<&Cell> {
        self.pos_to_ind(r_ind, c_ind)
            .and_then(|ind| self.cells.get(ind))
    }

    fn cell_mut(&mut self, r_ind: usize, c_ind: usize) -> Option<&mut Cell> {
        self.pos_to_ind(r_ind, c_ind)
            .and_then(|ind| self.cells.get_mut(ind))
    }

    fn pos_to_ind(&self, r_ind: usize, c_ind: usize) -> Option<usize> {
        if r_ind < self.height && c_ind < self.width {
            Some(r_ind * self.width + c_ind)
        } else {
            None
        }
    }
}

impl Display for Maze {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        AsciiMazeDisplay(self).fmt(f)
    }
}

#[derive(Debug, Clone, Default)]
struct Cell {
    is_connected_to_north: bool,
    is_connected_to_east: bool,
}

impl Cell {
    pub fn new() -> Self {
        Self::default()
    }
}
