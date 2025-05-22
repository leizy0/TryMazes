use std::{
    collections::{HashMap, HashSet, LinkedList},
    iter,
};

use rand::{Rng, seq::IteratorRandom};

use crate::maze::{Grid2d, Position2d};

pub mod circ;
pub mod hexa;
pub mod rect;
pub mod tri;

pub trait Maze2dGenerator {
    fn generate_2d(&self, grid: &mut dyn Grid2d);
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct AldousBroderMazeGenerator;

impl Maze2dGenerator for AldousBroderMazeGenerator {
    fn generate_2d(&self, grid: &mut dyn Grid2d) {
        let mut visited_pos = HashSet::new();
        let mut rng = rand::rng();
        let Some(mut cur_pos) = grid.random_cell_pos() else {
            // Empty grid
            return;
        };
        visited_pos.insert(cur_pos);
        let mut unvisited_cells_n = grid.cells_n() - 1;
        let mut neighbors = Vec::new();
        while unvisited_cells_n > 0 {
            neighbors.clear();
            grid.append_neighbors(&cur_pos, &mut neighbors);
            let candidate = neighbors
                .iter()
                .choose(&mut rng)
                .expect("There should be at least one neighbor in given non-empty grid.");
            if !visited_pos.contains(candidate) {
                grid.connect_to(&cur_pos, candidate);
                visited_pos.insert(*candidate);
                unvisited_cells_n -= 1;
            }

            cur_pos = *candidate;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct WilsonMazeGenerator;

impl Maze2dGenerator for WilsonMazeGenerator {
    fn generate_2d(&self, grid: &mut dyn Grid2d) {
        let mut unvisited_pos = grid.all_cells_pos_set();
        let mut rng = rand::rng();
        let Some(first_visited_pos) = unvisited_pos.iter().choose(&mut rng).copied() else {
            return;
        };
        unvisited_pos.remove(&first_visited_pos);
        let mut cur_path = Vec::new();
        let mut walk_visited_pos = HashMap::new();
        let mut neighbors = Vec::new();
        while !unvisited_pos.is_empty() {
            cur_path.clear();
            walk_visited_pos.clear();
            let Some(mut cur_pos) = unvisited_pos.iter().choose(&mut rng).copied() else {
                break;
            };
            loop {
                // Random walk.
                neighbors.clear();
                grid.append_neighbors(&cur_pos, &mut neighbors);
                let candidate = neighbors
                    .iter()
                    .choose(&mut rng)
                    .expect("There should be at least one neighbor in given non-empty grid.");
                walk_visited_pos.insert(cur_pos, cur_path.len());
                cur_path.push(cur_pos);
                if !unvisited_pos.contains(candidate) {
                    // Touch visited position in previous walks, path ends.
                    cur_path.push(*candidate);
                    break;
                }

                if let Some(path_ind) = walk_visited_pos.get(candidate).copied() {
                    // Remove loop.
                    for _ in path_ind..cur_path.len() {
                        let remove_pos = cur_path.pop().unwrap();
                        walk_visited_pos.remove(&remove_pos);
                    }
                }
                cur_pos = *candidate;
            }

            debug_assert!(cur_path.len() >= 2);
            for (from, to) in cur_path[0..cur_path.len().saturating_sub(1)]
                .iter()
                .zip(cur_path[1..].iter())
            {
                grid.connect_to(from, to);
                debug_assert!(unvisited_pos.contains(from));
                unvisited_pos.remove(from);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct HuntAndKillMazeGenerator;

impl Maze2dGenerator for HuntAndKillMazeGenerator {
    fn generate_2d(&self, grid: &mut dyn Grid2d) {
        let mut unvisited_pos = grid.all_cells_pos_set();
        let mut rng = rand::rng();
        let Some(mut cur_pos) = grid.random_cell_pos() else {
            return;
        };
        let mut neighbors = Vec::new();
        while !unvisited_pos.is_empty() {
            loop {
                unvisited_pos.remove(&cur_pos);
                // Select an unvisited candidate randomly.
                neighbors.clear();
                grid.append_neighbors(&cur_pos, &mut neighbors);
                let Some(candidate) = neighbors
                    .iter()
                    .filter(|neighbor| unvisited_pos.contains(neighbor))
                    .choose(&mut rng)
                else {
                    break;
                };
                grid.connect_to(&cur_pos, candidate);
                cur_pos = *candidate;
            }

            // // Hunt phase.
            let Some((next_start_pos, prev_visited_pos)) = unvisited_pos
                .iter()
                .filter_map(|pos| {
                    neighbors.clear();
                    grid.append_neighbors(pos, &mut neighbors);
                    neighbors
                        .iter()
                        .filter(|neighbor| !unvisited_pos.contains(neighbor))
                        .choose(&mut rng)
                        .map(|neighbor| (*pos, *neighbor))
                })
                .next()
            else {
                break;
            };
            grid.connect_to(&prev_visited_pos, &next_start_pos);
            cur_pos = next_start_pos;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct RecursiveBacktrackerMazeGenerator;

impl Maze2dGenerator for RecursiveBacktrackerMazeGenerator {
    fn generate_2d(&self, grid: &mut dyn Grid2d) {
        let mut rng = rand::rng();
        let Some(start_pos) = grid.random_cell_pos() else {
            return;
        };
        let mut visited_pos = HashSet::new();
        let mut unvisited_cells_n = grid.cells_n();
        let mut visited_stack = Vec::from_iter(iter::once(start_pos));
        let mut neighbors = Vec::new();
        while unvisited_cells_n > 0 {
            let Some(cur_pos) = visited_stack.last().copied() else {
                break;
            };
            if visited_pos.insert(cur_pos) {
                unvisited_cells_n -= 1;
            }

            // Select an unvisited candidate randomly.
            neighbors.clear();
            grid.append_neighbors(&cur_pos, &mut neighbors);
            let Some(candidate) = neighbors
                .iter()
                .filter(|neighbor| !visited_pos.contains(neighbor))
                .choose(&mut rng)
            else {
                // At dead end, try to backtrack.
                visited_stack.pop();
                continue;
            };
            grid.connect_to(&cur_pos, candidate);
            visited_stack.push(*candidate);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct MazeEdge {
    pub low: Position2d,
    pub high: Position2d,
}

impl MazeEdge {
    pub fn new(left: &Position2d, right: &Position2d) -> Self {
        Self {
            low: *left.min(right),
            high: *left.max(right),
        }
    }
}

struct Union {
    ele_inds: HashMap<Position2d, usize>,
    set_ids: Vec<usize>,
    sets_n: usize,
}

impl<'a> FromIterator<&'a Position2d> for Union {
    fn from_iter<T: IntoIterator<Item = &'a Position2d>>(iter: T) -> Self {
        let mut ele_inds = HashMap::new();
        for ele in iter.into_iter().cloned() {
            if !ele_inds.contains_key(&ele) {
                ele_inds.insert(ele, ele_inds.len());
            }
        }
        let set_ids: Vec<_> = (0..ele_inds.len()).collect();
        Self {
            ele_inds,
            sets_n: set_ids.len(),
            set_ids,
        }
    }
}

impl Union {
    pub fn sets_n(&self) -> usize {
        self.sets_n
    }

    pub fn merge(&mut self, ele0: &Position2d, ele1: &Position2d) -> bool {
        let Some(set_id0) = self.ele_set_id(ele0) else {
            return false;
        };
        let Some(set_id1) = self.ele_set_id(ele1) else {
            return false;
        };
        if set_id0 == set_id1 {
            return false;
        }

        let from_id = set_id0.min(set_id1);
        let to_id = set_id0.max(set_id1);
        self.set_ids[from_id] = to_id;
        self.sets_n -= 1;
        true
    }

    fn ele_set_id(&mut self, ele: &Position2d) -> Option<usize> {
        self.ele_inds.get(ele).copied().map(|ind| self.set_id(ind))
    }

    fn set_id(&mut self, ind: usize) -> usize {
        let parent_ind = self.set_ids[ind];
        if parent_ind == ind {
            return ind;
        }

        let root_ind = self.set_id(parent_ind);
        self.set_ids[ind] = root_ind;
        root_ind
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct KruskalMazeGenerator;

impl Maze2dGenerator for KruskalMazeGenerator {
    fn generate_2d(&self, grid: &mut dyn Grid2d) {
        let all_pos = grid.all_cells_pos_set();
        let mut neighbors = Vec::new();
        let mut edges = HashSet::new();
        for pos in all_pos.iter() {
            neighbors.clear();
            grid.append_neighbors(pos, &mut neighbors);
            edges.extend(
                neighbors
                    .iter()
                    .map(|neighbor| MazeEdge::new(pos, neighbor)),
            );
        }
        let mut rng = rand::rng();
        let mut union = Union::from_iter(all_pos.iter());
        while union.sets_n() > 1 {
            let Some(edge) = edges.iter().choose(&mut rng).cloned() else {
                break;
            };
            if union.merge(&edge.low, &edge.high) {
                grid.connect_to(&edge.low, &edge.high);
            }

            edges.remove(&edge);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct PrimMazeGenerator;

impl Maze2dGenerator for PrimMazeGenerator {
    fn generate_2d(&self, grid: &mut dyn Grid2d) {
        let cells_n = grid.cells_n();
        let Some(start_pos) = grid.random_cell_pos() else {
            return;
        };
        let mut neighbors = Vec::new();
        grid.append_neighbors(&start_pos, &mut neighbors);
        let mut edges: HashSet<_> = neighbors
            .iter()
            .map(|neighbor| MazeEdge::new(&start_pos, neighbor))
            .collect();
        let mut visited_pos: HashSet<_> = iter::once(start_pos).collect();
        let mut rng = rand::rng();
        while visited_pos.len() < cells_n {
            let Some(edge) = edges.iter().choose(&mut rng).cloned() else {
                break;
            };
            edges.remove(&edge);
            let (from, to) = if !visited_pos.contains(&edge.low) {
                debug_assert!(visited_pos.contains(&edge.high));
                (edge.high, edge.low)
            } else if !visited_pos.contains(&edge.high) {
                (edge.low, edge.high)
            } else {
                continue;
            };

            grid.connect_to(&from, &to);
            visited_pos.insert(to);
            neighbors.clear();
            grid.append_neighbors(&to, &mut neighbors);
            edges.extend(
                neighbors
                    .iter()
                    .filter(|neighbor| !visited_pos.contains(neighbor))
                    .map(|neighbor| MazeEdge::new(&to, neighbor)),
            );
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct GrowingTreeMazeGenerator;

impl Maze2dGenerator for GrowingTreeMazeGenerator {
    fn generate_2d(&self, grid: &mut dyn Grid2d) {
        let cells_n = grid.cells_n();
        let Some(start_pos) = grid.random_cell_pos() else {
            return;
        };
        let mut active_pos: LinkedList<_> = iter::once(start_pos).collect();
        let mut visited_pos: HashSet<_> = iter::once(start_pos).collect();
        let mut rng = rand::rng();
        let mut neighbors = Vec::new();
        while visited_pos.len() < cells_n {
            if active_pos.is_empty() {
                break;
            }
            let active_ind = rng.random_range(0..active_pos.len());
            let pos = *active_pos.iter().nth(active_ind).unwrap();
            neighbors.clear();
            grid.append_neighbors(&pos, &mut neighbors);
            let Some(neighbor) = neighbors
                .iter()
                .filter(|neighbor| !visited_pos.contains(neighbor))
                .choose(&mut rng)
            else {
                // No unvisited neighbor is available, so delete the selected active position
                let mut tail = active_pos.split_off(active_ind + 1);
                active_pos.pop_back();
                active_pos.append(&mut tail);
                continue;
            };

            active_pos.push_back(*neighbor);
            visited_pos.insert(*neighbor);
            grid.connect_to(&pos, neighbor);
        }
    }
}
