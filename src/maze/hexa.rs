use std::marker::PhantomData;

use super::{
    GeneralRectGrid, Grid2d, LayerGrid, MaskType, NoMask, Position2d, WithMask, rect::RectMask,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HexaDirection {
    North,
    NorthEast,
    SouthEast,
    South,
    SouthWest,
    NorthWest,
}

impl HexaDirection {
    fn all_dirs() -> &'static [HexaDirection] {
        static ALL_DIRECTIONS: [HexaDirection; 6] = [
            HexaDirection::North,
            HexaDirection::NorthEast,
            HexaDirection::SouthEast,
            HexaDirection::NorthWest,
            HexaDirection::SouthWest,
            HexaDirection::South,
        ];
        &ALL_DIRECTIONS
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HexaPosition {
    pub row: usize,
    pub col: usize,
}

impl From<Position2d> for HexaPosition {
    fn from(value: Position2d) -> Self {
        Self::new(value.0, value.1)
    }
}

impl From<HexaPosition> for Position2d {
    fn from(value: HexaPosition) -> Self {
        Self(value.row, value.col)
    }
}

impl HexaPosition {
    pub fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }

    pub fn neighbor(&self, dir: HexaDirection) -> Option<Self> {
        match dir {
            HexaDirection::North if self.row > 0 => Some(Self::new(self.row - 1, self.col)),
            HexaDirection::NorthEast if self.col % 2 == 0 && self.row > 0 => {
                Some(Self::new(self.row - 1, self.col + 1))
            }
            HexaDirection::NorthEast if self.col % 2 == 1 => {
                Some(Self::new(self.row, self.col + 1))
            }
            HexaDirection::SouthEast if self.col % 2 == 0 => {
                Some(Self::new(self.row, self.col + 1))
            }
            HexaDirection::SouthEast if self.col % 2 == 1 => {
                Some(Self::new(self.row + 1, self.col + 1))
            }
            HexaDirection::South => Some(Self::new(self.row + 1, self.col)),
            HexaDirection::SouthWest if self.col > 0 && self.col % 2 == 0 => {
                Some(Self::new(self.row, self.col - 1))
            }
            HexaDirection::SouthWest if self.col > 0 && self.col % 2 == 1 => {
                Some(Self::new(self.row + 1, self.col - 1))
            }
            HexaDirection::NorthWest if self.col > 0 && self.col % 2 == 0 && self.row > 0 => {
                Some(Self::new(self.row - 1, self.col - 1))
            }
            HexaDirection::NorthWest if self.col > 0 && self.col % 2 == 1 => {
                Some(Self::new(self.row, self.col - 1))
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
struct HexaCell {
    is_connected_to_north: bool,
    is_connected_to_northwest: bool,
    is_connected_to_southwest: bool,
}

#[derive(Debug, Clone)]
pub struct HexaGrid<M: MaskType>(GeneralRectGrid<HexaCell>, PhantomData<M>);

impl<M: MaskType> Grid2d for HexaGrid<M> {
    fn cells_n(&self) -> usize {
        self.0.cells_n()
    }

    fn random_cell_pos(&self) -> Option<Position2d> {
        self.0.random_cell_pos()
    }

    fn all_cells_pos_set(&self) -> std::collections::HashSet<Position2d> {
        self.0.all_cells_pos_set()
    }

    fn append_neighbors(&self, pos: &Position2d, neighbors: &mut Vec<Position2d>) {
        let pos = (*pos).into();
        neighbors.extend(
            HexaDirection::all_dirs()
                .iter()
                .flat_map(|dir| self.neighbor_pos(&pos, *dir))
                .map(Position2d::from),
        )
    }

    fn connect_to(&mut self, from: &Position2d, to: &Position2d) -> bool {
        let hex_from = (*from).into();
        let hex_to = (*to).into();
        let Some(dir) = HexaDirection::all_dirs()
            .iter()
            .find(|dir| {
                self.neighbor_pos(&hex_from, **dir)
                    .is_some_and(|neighbor| neighbor == hex_to)
            })
            .copied()
        else {
            return false;
        };
        match dir {
            HexaDirection::North => self.0.cell_mut(from).unwrap().is_connected_to_north = true,
            HexaDirection::NorthEast => {
                self.0.cell_mut(to).unwrap().is_connected_to_southwest = true
            }
            HexaDirection::SouthEast => {
                self.0.cell_mut(to).unwrap().is_connected_to_northwest = true
            }
            HexaDirection::South => self.0.cell_mut(to).unwrap().is_connected_to_north = true,
            HexaDirection::SouthWest => {
                self.0.cell_mut(from).unwrap().is_connected_to_southwest = true
            }
            HexaDirection::NorthWest => {
                self.0.cell_mut(from).unwrap().is_connected_to_northwest = true
            }
        }

        true
    }
}

impl LayerGrid for HexaGrid<NoMask> {
    fn layers_n(&self) -> usize {
        self.0.height
    }

    fn cells_n_at(&self, _layer_ind: usize) -> usize {
        self.0.width
    }

    fn append_neighbors_upper_layer(&self, pos: &Position2d, neighbors: &mut Vec<Position2d>) {
        neighbors.extend(
            self.neighbor_pos(&(*pos).into(), HexaDirection::North)
                .map(Position2d::from),
        )
    }

    fn append_neighbors_lower_layer(&self, pos: &Position2d, neighbors: &mut Vec<Position2d>) {
        neighbors.extend(
            self.neighbor_pos(&(*pos).into(), HexaDirection::South)
                .map(Position2d::from),
        )
    }
}

impl HexaGrid<NoMask> {
    pub fn new(width: usize, height: usize) -> Self {
        HexaGrid::<NoMask>(
            GeneralRectGrid::<HexaCell>::new(width, height),
            PhantomData::<NoMask>,
        )
    }
}

impl HexaGrid<WithMask> {
    pub fn new(mask: &RectMask) -> Self {
        HexaGrid::<WithMask>(
            GeneralRectGrid::<HexaCell>::with_mask(mask),
            PhantomData::<WithMask>,
        )
    }
}

impl<M: MaskType> HexaGrid<M> {
    pub fn neighbor_pos(
        &self,
        hexa_pos: &HexaPosition,
        dir: HexaDirection,
    ) -> Option<HexaPosition> {
        let pos = (*hexa_pos).into();
        if self.0.is_cell(&pos) {
            hexa_pos
                .neighbor(dir)
                .filter(|neighbor| self.0.is_cell(&(*neighbor).into()))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub enum HexaMaze {
    NoMask(HexaGrid<NoMask>),
    WithMask(HexaGrid<WithMask>),
}

impl HexaMaze {
    pub fn size(&self) -> (usize, usize) {
        self.grid().size()
    }

    pub fn is_cell(&self, pos: &HexaPosition) -> bool {
        self.grid().is_cell(&(*pos).into())
    }

    pub fn is_connected_to(&self, hexa_pos: &HexaPosition, dir: HexaDirection) -> bool {
        let pos = (*hexa_pos).into();
        let grid = self.grid();
        self.is_cell(hexa_pos)
            && match dir {
                direct_dir @ (HexaDirection::North
                | HexaDirection::SouthWest
                | HexaDirection::NorthWest) => {
                    let cell = grid.cell(&pos).unwrap();
                    match direct_dir {
                        HexaDirection::North => cell.is_connected_to_north,
                        HexaDirection::SouthWest => cell.is_connected_to_southwest,
                        HexaDirection::NorthWest => cell.is_connected_to_northwest,
                        _ => unreachable!(),
                    }
                }
                indirect_dir @ (HexaDirection::NorthEast
                | HexaDirection::SouthEast
                | HexaDirection::South) => {
                    hexa_pos.neighbor(indirect_dir).is_some_and(|neighbor| {
                        grid.cell(&neighbor.into())
                            .is_some_and(|cell| match indirect_dir {
                                HexaDirection::NorthEast => cell.is_connected_to_southwest,
                                HexaDirection::SouthEast => cell.is_connected_to_northwest,
                                HexaDirection::South => cell.is_connected_to_north,
                                _ => unreachable!(),
                            })
                    })
                }
            }
    }

    fn grid(&self) -> &GeneralRectGrid<HexaCell> {
        match self {
            HexaMaze::NoMask(hexa_grid) => &hexa_grid.0,
            HexaMaze::WithMask(hexa_grid) => &hexa_grid.0,
        }
    }
}
