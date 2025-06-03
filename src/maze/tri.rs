use serde::{Deserialize, Serialize};

use super::{DefaultInRectGrid, GeneralRectGrid, Grid2d, Position2d, rect::RectMask};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TriDirection {
    Northwest,
    Northeast,
    South,
    SouthWest,
    North,
    Southeast,
}

impl TriDirection {
    pub fn angle_up_all_dirs() -> &'static [Self] {
        static ANGLE_UP_ALL_DIRECTIONS: [TriDirection; 3] = [
            TriDirection::Northwest,
            TriDirection::Northeast,
            TriDirection::South,
        ];
        &ANGLE_UP_ALL_DIRECTIONS
    }

    pub fn angle_down_all_dirs() -> &'static [Self] {
        static ANGLE_DOWN_ALL_DIRECTIONS: [TriDirection; 3] = [
            TriDirection::SouthWest,
            TriDirection::North,
            TriDirection::Southeast,
        ];
        &ANGLE_DOWN_ALL_DIRECTIONS
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TriPosition {
    pub row: usize,
    pub col: usize,
}

impl From<Position2d> for TriPosition {
    fn from(value: Position2d) -> Self {
        Self::new(value.0, value.1)
    }
}

impl From<TriPosition> for Position2d {
    fn from(value: TriPosition) -> Self {
        Position2d(value.row, value.col)
    }
}

impl TriPosition {
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }

    pub fn neighbor(&self, dir: TriDirection) -> Option<Self> {
        match dir {
            TriDirection::Northwest | TriDirection::SouthWest if self.col > 0 => {
                Some(Self::new(self.row, self.col - 1))
            }
            TriDirection::Northeast | TriDirection::Southeast => {
                Some(Self::new(self.row, self.col + 1))
            }
            TriDirection::South => Some(Self::new(self.row + 1, self.col)),
            TriDirection::North if self.row > 0 => Some(Self::new(self.row - 1, self.col)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum TriCell {
    #[serde(rename = "u")]
    AngelUp {
        #[serde(rename = "nw")]
        is_connected_to_northwest: bool,
        #[serde(rename = "s")]
        is_connected_to_south: bool,
    },
    #[serde(rename = "d")]
    AngelDown {
        #[serde(rename = "sw")]
        is_connected_to_southwest: bool,
    },
}

impl DefaultInRectGrid for TriCell {
    fn default_at(pos: &Position2d) -> Self {
        if (pos.0 + pos.1) % 2 == 0 {
            Self::AngelUp {
                is_connected_to_northwest: false,
                is_connected_to_south: false,
            }
        } else {
            Self::AngelDown {
                is_connected_to_southwest: false,
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriGrid(GeneralRectGrid<TriCell>);

impl Grid2d for TriGrid {
    fn cells_n(&self) -> usize {
        self.0.cells_n()
    }

    fn random_cell_pos(&self) -> Option<super::Position2d> {
        self.0.random_cell_pos()
    }

    fn all_cells_pos_set(&self) -> std::collections::HashSet<super::Position2d> {
        self.0.all_cells_pos_set()
    }

    fn append_neighbors(&self, pos: &super::Position2d, neighbors: &mut Vec<super::Position2d>) {
        let tri_pos: TriPosition = (*pos).into();
        neighbors.extend(self.0.cell(pos).into_iter().flat_map(|cell| {
            match cell {
                TriCell::AngelUp { .. } => TriDirection::angle_up_all_dirs(),
                TriCell::AngelDown { .. } => TriDirection::angle_down_all_dirs(),
            }
            .iter()
            .flat_map(|dir| self.neighbor_pos(&tri_pos, *dir))
            .map(Position2d::from)
        }));
    }

    fn connect_to(&mut self, from: &super::Position2d, to: &super::Position2d) -> bool {
        let tri_from = (*from).into();
        let tri_to = (*to).into();
        let Some(dir) = self.0.cell(from).and_then(|cell| {
            match cell {
                TriCell::AngelUp { .. } => TriDirection::angle_up_all_dirs(),
                TriCell::AngelDown { .. } => TriDirection::angle_down_all_dirs(),
            }
            .iter()
            .find(|dir| {
                self.neighbor_pos(&tri_from, **dir)
                    .is_some_and(|neighbor| neighbor == tri_to)
            })
        }) else {
            return false;
        };

        match dir {
            TriDirection::Northwest => match self.0.cell_mut(from).unwrap() {
                TriCell::AngelUp {
                    is_connected_to_northwest,
                    ..
                } => *is_connected_to_northwest = true,
                TriCell::AngelDown { .. } => unreachable!(),
            },
            TriDirection::Northeast => match self.0.cell_mut(to).unwrap() {
                TriCell::AngelUp { .. } => unreachable!(),
                TriCell::AngelDown {
                    is_connected_to_southwest,
                } => *is_connected_to_southwest = true,
            },
            TriDirection::South => match self.0.cell_mut(from).unwrap() {
                TriCell::AngelUp {
                    is_connected_to_south,
                    ..
                } => *is_connected_to_south = true,
                TriCell::AngelDown { .. } => unreachable!(),
            },
            TriDirection::SouthWest => match self.0.cell_mut(from).unwrap() {
                TriCell::AngelUp { .. } => unreachable!(),
                TriCell::AngelDown {
                    is_connected_to_southwest,
                } => *is_connected_to_southwest = true,
            },
            TriDirection::North => match self.0.cell_mut(to).unwrap() {
                TriCell::AngelUp {
                    is_connected_to_south,
                    ..
                } => *is_connected_to_south = true,
                TriCell::AngelDown { .. } => unreachable!(),
            },
            TriDirection::Southeast => match self.0.cell_mut(to).unwrap() {
                TriCell::AngelUp {
                    is_connected_to_northwest,
                    ..
                } => *is_connected_to_northwest = true,
                TriCell::AngelDown { .. } => unreachable!(),
            },
        }

        true
    }
}

impl TriGrid {
    pub fn new(width: usize, height: usize) -> Self {
        Self(GeneralRectGrid::new(width, height))
    }

    pub fn with_mask(mask: &RectMask) -> Self {
        Self(GeneralRectGrid::with_mask(mask))
    }

    pub fn neighbor_pos(&self, pos: &TriPosition, dir: TriDirection) -> Option<TriPosition> {
        self.0.cell(&(*pos).into()).and_then(|cell| {
            match cell {
                TriCell::AngelUp { .. } => TriDirection::angle_up_all_dirs(),
                TriCell::AngelDown { .. } => TriDirection::angle_down_all_dirs(),
            }
            .iter()
            .find(|neighbor_dir| **neighbor_dir == dir)
            .and_then(|dir| {
                pos.neighbor(*dir)
                    .filter(|neighbor| self.0.is_cell(&(*neighbor).into()))
            })
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriMaze(TriGrid);

impl TriMaze {
    pub fn new(grid: TriGrid) -> Self {
        Self(grid)
    }

    pub fn size(&self) -> (usize, usize) {
        self.0.0.size()
    }

    pub fn is_cell(&self, pos: &TriPosition) -> bool {
        self.0.0.is_cell(&(*pos).into())
    }

    pub fn is_connected_to(&self, tri_pos: &TriPosition, dir: TriDirection) -> bool {
        let pos = (*tri_pos).into();
        self.0.0.cell(&pos).is_some_and(|cell| match cell {
            TriCell::AngelUp {
                is_connected_to_northwest,
                is_connected_to_south,
            } => match dir {
                TriDirection::Northwest => *is_connected_to_northwest,
                TriDirection::South => *is_connected_to_south,
                TriDirection::Northeast => self
                    .0
                    .neighbor_pos(tri_pos, TriDirection::Northeast)
                    .is_some_and(|neighbor| {
                        self.is_connected_to(&neighbor, TriDirection::SouthWest)
                    }),
                _ => false,
            },
            TriCell::AngelDown {
                is_connected_to_southwest,
            } => match dir {
                TriDirection::SouthWest => *is_connected_to_southwest,
                TriDirection::North => self
                    .0
                    .neighbor_pos(tri_pos, TriDirection::North)
                    .is_some_and(|neighbor| self.is_connected_to(&neighbor, TriDirection::South)),
                TriDirection::Southeast => self
                    .0
                    .neighbor_pos(tri_pos, TriDirection::Southeast)
                    .is_some_and(|neighbor| {
                        self.is_connected_to(&neighbor, TriDirection::Northwest)
                    }),
                _ => false,
            },
        })
    }

    pub fn is_angle_up(&self, pos: &TriPosition) -> bool {
        match TriCell::default_at(&(*pos).into()) {
            TriCell::AngelUp { .. } => true,
            TriCell::AngelDown { .. } => false,
        }
    }
}
