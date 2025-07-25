use std::{
    cell::RefCell,
    collections::{HashMap, HashSet, LinkedList},
    hash::Hash,
    iter,
};

use rand::{Rng, seq::IteratorRandom};

use crate::maze::{Grid2d, LayerGrid, Position2d};

pub mod circ;
pub mod hexa;
pub mod rect;
pub mod tri;

/// The most general generator for maze in 2D.
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
                    // The current path has a loop(visit this repeated position twice), remove it by cutting the tail(including the repeated position).
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
                // Connect the current path.
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
            // Kill phase.
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

            // Hunt phase, find a position which neighbors contain a visited position.
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

// A structure to represent neighbor relationship between positions in maze.
// Using the order of position to ensure that there's only one edge between two neighbors.
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

/// A helper to find which set an element belongs after some arbitrary union operations. Every element starts from a set only contains itself.
struct Union<T: Hash + Eq> {
    ele_inds: HashMap<T, usize>,
    set_ids: RefCell<Vec<usize>>,
    sets_n: usize,
}

impl<T: Hash + Eq> FromIterator<T> for Union<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut new_union = Union::new();
        for ele in iter.into_iter() {
            new_union.add(ele);
        }
        new_union
    }
}

impl<T: Hash + Eq> Union<T> {
    pub fn new() -> Self {
        Self {
            ele_inds: HashMap::new(),
            set_ids: RefCell::new(Vec::new()),
            sets_n: 0,
        }
    }

    pub fn sets_n(&self) -> usize {
        self.sets_n
    }

    pub fn add(&mut self, ele: T) {
        if self.ele_inds.contains_key(&ele) {
            return;
        }

        debug_assert!(self.ele_inds.len() == self.set_ids.borrow().len());
        let ind = self.ele_inds.len();
        self.ele_inds.insert(ele, ind);
        self.set_ids.borrow_mut().push(ind);
        self.sets_n += 1;
    }

    pub fn merge(&mut self, ele0: &T, ele1: &T) -> bool {
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
        self.set_ids.borrow_mut()[from_id] = to_id;
        self.sets_n -= 1;
        true
    }

    pub fn into_sets(self) -> Vec<HashSet<T>> {
        let set_id_to_ind: HashMap<_, _> = self
            .set_ids
            .borrow()
            .iter()
            .enumerate()
            .filter(|(ele_ind, id)| ele_ind == *id)
            .map(|(_, id)| *id)
            .enumerate()
            .map(|(set_ind, id)| (id, set_ind))
            .collect();
        let mut sets: Vec<_> = iter::repeat_with(|| HashSet::new())
            .take(set_id_to_ind.len())
            .collect();
        for (pos, ind) in self.ele_inds {
            let set_ind = set_id_to_ind[&Self::set_id(&self.set_ids, ind)];
            sets[set_ind].insert(pos);
        }

        sets
    }

    fn is_union(&self, ele0: &T, ele1: &T) -> Option<bool> {
        self.ele_set_id(ele0)
            .and_then(|set_ind0| self.ele_set_id(ele1).map(|set_ind1| set_ind0 == set_ind1))
    }

    fn ele_set_id(&self, ele: &T) -> Option<usize> {
        self.ele_inds
            .get(ele)
            .copied()
            .map(|ind| Self::set_id(&self.set_ids, ind))
    }

    fn set_id(set_ids: &RefCell<Vec<usize>>, ind: usize) -> usize {
        let parent_ind = set_ids.borrow()[ind];
        if parent_ind == ind {
            return ind;
        }

        let root_ind = Self::set_id(set_ids, parent_ind);
        set_ids.borrow_mut()[ind] = root_ind;
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
        // Find and save all neighbors(edges) in the maze.
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
        let mut cell_pos_union = Union::from_iter(all_pos);
        while cell_pos_union.sets_n() > 1 {
            // Select an edge randomly.
            let Some(edge) = edges.iter().choose(&mut rng).cloned() else {
                break;
            };
            if cell_pos_union.merge(&edge.low, &edge.high) {
                // The edge can connect two different areas in the current maze, connect it to merge these two areas.
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
            // Select an edge randomly.
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
                // The two positions in the edge all have been visited, ignore it.
                continue;
            };

            // The edge can connect to an unvisited position, so connect it.
            grid.connect_to(&from, &to);
            visited_pos.insert(to);
            // Add edges which have unvisited neighbor to candidates.
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
            // Select an active position randomly.
            let active_ind = rng.random_range(0..active_pos.len());
            let pos = *active_pos.iter().nth(active_ind).unwrap();
            // Select an unvisited neighbor randomly.
            neighbors.clear();
            grid.append_neighbors(&pos, &mut neighbors);
            let Some(neighbor) = neighbors
                .iter()
                .filter(|neighbor| !visited_pos.contains(neighbor))
                .choose(&mut rng)
            else {
                // No unvisited neighbor is available, so remove the selected active position
                let mut tail = active_pos.split_off(active_ind + 1);
                active_pos.pop_back();
                active_pos.append(&mut tail);
                continue;
            };

            // Mark the neighbor as a new active position, and connect to it.
            active_pos.push_back(*neighbor);
            visited_pos.insert(*neighbor);
            grid.connect_to(&pos, neighbor);
        }
    }
}

pub trait LayerMazeGenerator {
    fn generate_layer(&self, grid: &mut dyn LayerGrid);
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct EllerMazeGenerator;

impl LayerMazeGenerator for EllerMazeGenerator {
    fn generate_layer(&self, grid: &mut dyn LayerGrid) {
        let layers_n = grid.layers_n();
        let mut layer_union = Union::new();
        let mut rng = rand::rng();
        let mut lower_neighbors = Vec::new();
        for layer_ind in 0..layers_n {
            let layer_cells_n = grid.cells_n_at(layer_ind);
            for cell_ind in 0..layer_cells_n {
                let pos = Position2d(layer_ind, cell_ind);
                layer_union.add(pos);
                let Some(last_neighbor) = grid.last_neighbor_pos(&pos) else {
                    continue;
                };
                layer_union.add(last_neighbor);
                // If these two positions aren't in the same set, they can be connected,
                // if the current layer is the last layer, they should be connected, otherwise, there will be some unconnected area in the final maze.
                // At the other layers, they will be connected randomly with a probability of 1/2.
                let should_connect = !layer_union.is_union(&pos, &last_neighbor).unwrap()
                    && (layer_ind == layers_n - 1 || rng.random_ratio(1, 2));
                if should_connect {
                    // If connected, the sets they belongs to also should be united.
                    grid.connect_to(&last_neighbor, &pos);
                    layer_union.merge(&pos, &last_neighbor);
                }
            }

            if layer_ind == layers_n - 1 {
                // There's no next layer, exit.
                break;
            }

            let mut next_layer_union = Union::new();
            // Get the final sets in which the positions are connected in the current layer.
            let sets = layer_union.into_sets();
            for set in sets {
                // Randomly choose a position to connect to next layer, to ensure at least one position in the current set connects to the next layer.
                let dig_pos = set.iter().choose(&mut rng).cloned().unwrap();
                let mut first_dig_target = None;
                for pos in set.into_iter() {
                    // Positions which aren't chosen also can connect to the next layer randomly with a probability of 1/3.
                    let should_dig = pos == dig_pos || rng.random_ratio(1, 3);
                    if should_dig {
                        // Randomly chosen a neighbor in the next layer, and connect to it.
                        lower_neighbors.clear();
                        grid.append_neighbors_lower_layer(&pos, &mut lower_neighbors);
                        let dig_neighbor = lower_neighbors.iter().choose(&mut rng).unwrap();
                        grid.connect_to(&pos, dig_neighbor);
                        // Add the connected target neighbor in the current set to the same set.
                        next_layer_union.add(*dig_neighbor);
                        next_layer_union
                            .merge(dig_neighbor, first_dig_target.get_or_insert(*dig_neighbor));
                    }
                }
            }

            // The initial connection settings in the next layer.
            layer_union = next_layer_union;
        }
    }
}
