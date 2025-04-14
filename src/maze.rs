use std::fmt::Display;

use rand::Rng;

use crate::show::AsciiMazeDisplay;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    pub fn all_dirs() -> &'static [Direction; 4] {
        static ALL_DIRECTIONS: [Direction; 4] = [
            Direction::North,
            Direction::East,
            Direction::West,
            Direction::South,
        ];

        &ALL_DIRECTIONS
    }

    pub fn reverse(&self) -> Self {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Position {
    pub r: usize,
    pub c: usize,
}

impl Position {
    pub fn new(r: usize, c: usize) -> Self {
        Self { r, c }
    }

    pub fn random(rng: &mut impl Rng, width: usize, height: usize) -> Self {
        Self {
            r: rng.random_range(0..height),
            c: rng.random_range(0..width),
        }
    }

    pub fn neighbor(&self, dir: Direction) -> Option<Self> {
        match dir {
            Direction::North if self.r > 0 => Some(Position::new(self.r - 1, self.c)),
            Direction::South => Some(Position::new(self.r + 1, self.c)),
            Direction::East => Some(Position::new(self.r, self.c + 1)),
            Direction::West if self.c > 0 => Some(Position::new(self.r, self.c - 1)),
            _ => None,
        }
    }
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

    pub fn is_at_border(&self, pos: &Position, dir: Direction) -> bool {
        self.cell(pos).is_some()
            && match dir {
                Direction::North => pos.r == 0,
                Direction::South => pos.r == self.height - 1,
                Direction::East => pos.c == self.width - 1,
                Direction::West => pos.c == 0,
            }
    }

    pub fn is_connect_to(&self, pos: &Position, dir: Direction) -> bool {
        if let Some(cell) = self.cell(pos) {
            match dir {
                Direction::North => cell.is_connected_to_north,
                Direction::East => cell.is_connected_to_east,
                other_dir => pos
                    .neighbor(other_dir)
                    .is_some_and(|neighbor| self.is_connect_to(&neighbor, dir.reverse())),
            }
        } else {
            false
        }
    }

    pub fn neighbor_pos(&self, pos: &Position, dir: Direction) -> Option<Position> {
        pos.neighbor(dir)
            .filter(|neighbor| self.cell(neighbor).is_some())
    }

    pub fn connect_to(&mut self, pos: &Position, dir: Direction) {
        if let Some(cell) = self.cell_mut(pos) {
            match dir {
                Direction::North => cell.is_connected_to_north = true,
                Direction::East => cell.is_connected_to_east = true,
                other_dir => {
                    if let Some(neighbor) = pos.neighbor(other_dir) {
                        self.connect_to(&neighbor, dir.reverse());
                    }
                }
            }
        }
    }

    fn cell(&self, pos: &Position) -> Option<&Cell> {
        self.pos_to_ind(pos).and_then(|ind| self.cells.get(ind))
    }

    fn cell_mut(&mut self, pos: &Position) -> Option<&mut Cell> {
        self.pos_to_ind(pos).and_then(|ind| self.cells.get_mut(ind))
    }

    fn pos_to_ind(&self, pos: &Position) -> Option<usize> {
        if pos.r < self.height && pos.c < self.width {
            Some(pos.r * self.width + pos.c)
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
