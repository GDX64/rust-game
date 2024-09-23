use cgmath::{Vector2, Vector3};
use hierarchical_pathfinding::{prelude::MooreNeighborhood, PathCache, PathCacheConfig};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::{self, Debug, Formatter},
    vec,
};
use wasm_bindgen::prelude::*;

use crate::{
    island::{Island, IslandData, IslandTile},
    utils::{spiral_search::SpiralSearch, vectors::V2D},
};

const MIN_ISLAND_SIZE: usize = 300;
const LAND_WALK_COST: isize = -1;

impl Default for WorldGrid {
    fn default() -> Self {
        Self {
            dim: 0.0,
            tiles_dim: 0,
            tile_size: 1.0,
            data: Vec::new(),
            islands: BTreeMap::new(),
            path_cache: None,
            total_island_tiles: 0,
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy, PartialEq)]
pub enum TileKind {
    Water,
    Land,
    Forest,
    Lighthouse,
    Coast,
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            kind: TileKind::Water,
            height: 0.0,
            visited: false,
            island_number: None,
        }
    }
}

impl Tile {
    pub fn new(kind: TileKind, height: f64) -> Self {
        Self {
            kind,
            height,
            visited: false,
            island_number: None,
        }
    }

    pub fn grass(height: f64) -> Self {
        Self {
            kind: TileKind::Land,
            height,
            visited: false,
            island_number: None,
        }
    }

    pub fn kind(&self) -> TileKind {
        self.kind
    }

    pub fn mark_visited(&mut self) {
        self.visited = true;
    }

    pub fn was_visited(&self) -> bool {
        self.visited
    }

    pub fn is_water(&self) -> bool {
        self.kind == TileKind::Water || self.kind == TileKind::Lighthouse
    }

    pub fn is_land(&self) -> bool {
        self.kind == TileKind::Land || self.kind == TileKind::Forest
    }

    pub fn can_go(&self) -> bool {
        self.kind == TileKind::Water
    }

    pub fn height(&self) -> f64 {
        self.height
    }
}

#[derive(Clone, Copy)]
pub struct Tile {
    pub kind: TileKind,
    height: f64,
    visited: bool,
    pub island_number: Option<u64>,
}

pub struct TileUnit(usize);

impl TileUnit {
    pub fn div(val: f64, tile_size: f64) -> Self {
        Self((val / tile_size).floor() as usize)
    }
}

pub struct WorldGrid {
    /// This is the total size of the grid in meters.
    /// Thre grid area is dim*dim
    pub dim: f64,
    pub tiles_dim: usize,
    pub tile_size: f64,
    pub data: Vec<Tile>,
    pub islands: BTreeMap<u64, Island>,
    pub path_cache: Option<PathCache<MooreNeighborhood>>,
    pub total_island_tiles: usize,
}

impl WorldGrid {
    fn tile_unit(&self, val: f64) -> usize {
        TileUnit::div(val + self.dim / 2.0, self.tile_size).0
    }

    fn from_tile_unit(&self, unit: usize) -> f64 {
        (unit as f64 * self.tile_size) - self.dim / 2.0
    }

    pub fn get(&self, x: f64, y: f64) -> Option<&Tile> {
        let x = self.tile_unit(x);
        let y = self.tile_unit(y);
        return self.get_usize(x, y);
    }

    fn get_usize(&self, x: usize, y: usize) -> Option<&Tile> {
        let index = (y * self.tiles_dim + x) as usize;
        self.data.get(index)
    }

    fn get_i32(&self, x: i32, y: i32) -> Option<&Tile> {
        if x < 0 || y < 0 {
            return None;
        }
        let x = x as usize;
        let y = y as usize;
        let index = (y * self.tiles_dim + x) as usize;
        self.data.get(index)
    }

    pub fn new(dim: f64, default: Tile, tile_size: f64) -> Self {
        let tiles_dim = TileUnit::div(dim, tile_size).0;

        Self {
            dim,
            tiles_dim,
            tile_size,
            data: vec![default; tiles_dim * tiles_dim],
            islands: BTreeMap::new(),
            path_cache: None,
            total_island_tiles: 0,
        }
    }

    fn fill_coast(&mut self) {
        for y in 0..self.tiles_dim {
            for x in 0..self.tiles_dim {
                let index = y * self.tiles_dim + x;
                let tile = self.data.get(index).unwrap();
                if !tile.is_water() {
                    continue;
                }
                let search = SpiralSearch::new((x as i32, y as i32));
                for (x, y) in search.take(9) {
                    if let Some(tile) = self.get_i32(x, y) {
                        if tile.is_land() {
                            self.data[index] = Tile::new(TileKind::Coast, tile.height());
                            break;
                        }
                    }
                }
            }
        }
    }

    fn flood_fill_land(
        &mut self,
        x: i32,
        y: i32,
        island: u64,
        water_set: &mut BTreeSet<(i32, i32)>,
    ) -> BTreeSet<IslandTile> {
        let mut stack = vec![(x, y)];
        let mut set = BTreeSet::new();
        while let Some((x, y)) = stack.pop() {
            if x < 0 || y < 0 {
                continue;
            }
            let index = y * (self.tiles_dim as i32) + x;
            if let Some(tile) = self.data.get_mut(index as usize) {
                if tile.was_visited() {
                    continue;
                }
                if tile.is_land() {
                    tile.mark_visited();
                    tile.island_number = Some(island);

                    let island_tile = IslandTile::new(tile.height(), x, y);
                    set.insert(island_tile);

                    stack.push((x + 1, y));
                    stack.push((x - 1, y));
                    stack.push((x, y + 1));
                    stack.push((x, y - 1));
                } else {
                    water_set.insert((x, y));
                }
            }
        }
        return set;
    }

    pub fn find_islands(&mut self) {
        let x = 0i32;
        let y = 0i32;
        let mut water_stack = BTreeSet::new();
        water_stack.insert((x, y));
        let mut islands_number = 0;
        let mut island_map = BTreeMap::new();

        while let Some((x, y)) = water_stack.pop_first() {
            if x < 0 || y < 0 {
                continue;
            }
            let index = (y * (self.tiles_dim as i32) + x) as usize;
            if let Some(tile) = self.data.get_mut(index) {
                if tile.was_visited() {
                    continue;
                }
                if tile.is_water() {
                    tile.mark_visited();
                    water_stack.insert((x + 1, y));
                    water_stack.insert((x - 1, y));
                    water_stack.insert((x, y + 1));
                    water_stack.insert((x, y - 1));
                } else {
                    let set = self.flood_fill_land(x, y, islands_number, &mut water_stack);
                    if set.len() > MIN_ISLAND_SIZE {
                        let mut island = Island::new(set, islands_number, self.tile_size);
                        let center = island.calc_center();
                        let x = self.from_tile_unit(center.x as usize);
                        let y = self.from_tile_unit(center.y as usize);
                        island.center = V2D::new(x, y);
                        island_map.insert(islands_number, island);
                        islands_number += 1;
                    }
                }
            }
        }

        self.islands = island_map;
        self.fill_coast();
        self.fill_light_houses();
        self.calc_path_cache();
        self.total_island_tiles = 0;
        self.islands.values().for_each(|island| {
            self.total_island_tiles += island.tiles.len();
        });
    }

    fn fill_light_houses(&mut self) {
        let mut islands = std::mem::take(&mut self.islands);
        for island in islands.values_mut() {
            let light_house = self.find_lighthouse_place(&island);
            if let Some(light_house) = light_house {
                island.light_house = light_house;
                if let Some(tile) = self.get_mut(light_house.x, light_house.y) {
                    *tile = Tile::new(TileKind::Lighthouse, 0.0);
                }
            }
        }
        self.islands = islands;
    }

    pub fn is_forbidden_land(&self, x: f64, y: f64) -> bool {
        if x.abs() > self.dim / 2.0 || y.abs() > self.dim / 2.0 {
            return true;
        }
        let x = self.tile_unit(x);
        let y = self.tile_unit(y);
        if let Some(tile) = self.get_usize(x, y) {
            return tile.is_land();
        }
        return true;
    }

    fn calc_path_cache(&mut self) {
        let tiles_dim = self.tiles_dim;
        let path_cache = PathCache::new(
            (tiles_dim, tiles_dim), // the size of the Grid
            |(x, y)| {
                return self.walk_cost(x, y);
            }, // get the cost for walking over a Tile
            MooreNeighborhood::new(tiles_dim, tiles_dim), // the Neighborhood
            PathCacheConfig::with_chunk_size(70), // config
        );
        self.path_cache = Some(path_cache);
    }

    pub fn spiral_search(
        &self,
        x: f64,
        y: f64,
        mut is_ok: impl FnMut(f64, f64, &Tile) -> bool,
    ) -> Option<(f64, f64)> {
        let spiral = SpiralSearch::new((self.tile_unit(x) as i32, self.tile_unit(y) as i32));
        for (x, y) in spiral.take(10_000) {
            let x = self.from_tile_unit(x as usize);
            let y = self.from_tile_unit(y as usize);
            if is_ok(x, y, self.get(x, y)?) {
                return Some((x, y));
            }
        }
        return None;
    }

    fn is_surounded_by(&self, x: usize, y: usize, kind: TileKind) -> bool {
        let search = SpiralSearch::new((x as i32, y as i32));
        for (x, y) in search.take(25) {
            if let Some(tile) = self.get_usize(x as usize, y as usize) {
                if tile.kind() != kind {
                    return false;
                }
            }
        }
        return true;
    }

    fn find_lighthouse_place(&self, island: &Island) -> Option<V2D> {
        let center_x = self.tile_unit(island.center.x) as i32;
        let center_y = self.tile_unit(island.center.y) as i32;
        for (x, y) in SpiralSearch::new((center_x, center_y)).take(10_000) {
            let x = x as usize;
            let y = y as usize;
            if let Some(tile) = self.get_usize(x, y) {
                if tile.is_water() && self.is_surounded_by(x as usize, y as usize, TileKind::Water)
                {
                    let x = self.from_tile_unit(x as usize);
                    let y = self.from_tile_unit(y as usize);
                    return Some(V2D::new(x, y));
                }
            }
        }
        return None;
    }

    pub fn iter(&mut self) -> impl Iterator<Item = (f64, f64, &Tile)> {
        self.data.iter().enumerate().map(|(index, value)| {
            let x = index % self.tiles_dim;
            let y = index / self.tiles_dim;
            let x = self.from_tile_unit(x);
            let y = self.from_tile_unit(y);
            (x, y, value)
        })
    }

    pub fn get_mut(&mut self, x: f64, y: f64) -> Option<&mut Tile> {
        let x = self.tile_unit(x);
        let y = self.tile_unit(y);
        let index = (y * self.tiles_dim + x) as usize;
        self.data.get_mut(index)
    }

    pub fn set(&mut self, x: f64, y: f64, value: Tile) -> Option<()> {
        let x = self.tile_unit(x);
        let y = self.tile_unit(y);
        let index = y * self.tiles_dim + x;
        let p = self.data.get_mut(index)?;
        *p = value;
        Some(())
    }

    pub fn is_allowed_place(&self, x: f64, y: f64) -> bool {
        let x = self.tile_unit(x);
        let y = self.tile_unit(y);
        if let Some(value) = self.get_usize(x, y) {
            return value.can_go();
        }
        return false;
    }

    pub fn height_of(&self, x: f64, y: f64) -> f64 {
        let x = self.tile_unit(x);
        let y = self.tile_unit(y);
        if let Some(value) = self.get_usize(x, y) {
            return value.height();
        }
        return 0.0;
    }

    pub fn all_island_data(&self) -> Vec<IslandData> {
        self.islands
            .values()
            .filter_map(|x| self.island_data(x.id))
            .collect()
    }

    pub fn island_data(&self, id: u64) -> Option<IslandData> {
        return Some(self.islands.get(&id)?.island_data());
    }

    pub fn island_at(&self, x: f64, y: f64) -> Option<IslandData> {
        let x = self.tile_unit(x);
        let y = self.tile_unit(y);
        let tile = self.get_usize(x, y)?;
        let island = self.islands.get(&tile.island_number?)?;
        self.island_data(island.id)
    }

    fn can_go_straight(&self, initial: &V2D, fin: &V2D) -> bool {
        let mut line =
            grid_line::GridLinePath::new(initial.clone(), fin.clone(), self.tile_size / 2.0);
        while let Some(point) = line.next() {
            if !self.is_allowed_place(point.x, point.y) {
                return false;
            }
        }
        return true;
    }

    fn walk_cost(&self, x: usize, y: usize) -> isize {
        self.get_usize(x, y)
            .map(|tile| if tile.is_water() { 1 } else { LAND_WALK_COST })
            .unwrap_or(LAND_WALK_COST)
    }

    pub fn find_path(&self, initial: impl Into<V2D>, fin: impl Into<V2D>) -> Option<Vec<V2D>> {
        let initial = initial.into();
        let fin = fin.into();
        if fin.x.abs() > self.dim / 2.0 || fin.y.abs() > self.dim / 2.0 {
            return None;
        }
        if self.can_go_straight(&initial, &fin) {
            return Some(vec![initial.into(), fin.into()]);
        }

        let initial = Vector2::new(self.tile_unit(initial.x), self.tile_unit(initial.y));
        let fin = Vector2::new(self.tile_unit(fin.x), self.tile_unit(fin.y));
        let is_target_water = self.get_usize(fin.x as usize, fin.y as usize)?.is_water();
        if !is_target_water {
            return None;
        }

        let path_cache = self.path_cache.as_ref()?;
        let path = path_cache.find_path(initial.into(), fin.into(), |(x, y)| {
            return self.walk_cost(x, y);
        })?;

        let v: Vec<Vector2<f64>> = path
            .take(MAX_SEARCH)
            .map(|(x, y)| {
                let half_tile = self.tile_size / 2.0;
                Vector2::new(
                    self.from_tile_unit(x) + half_tile,
                    self.from_tile_unit(y) + half_tile,
                )
            })
            .collect();
        return Some(v);
    }
}

impl Debug for WorldGrid {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut s = String::new();
        for y in 0..self.tiles_dim {
            for x in 0..self.tiles_dim {
                let tile = self.get_usize(x, y).unwrap();
                let c = match tile.kind() {
                    TileKind::Water => "W ",
                    TileKind::Land => "G ",
                    TileKind::Forest => "F ",
                    TileKind::Lighthouse => "L ",
                    TileKind::Coast => "C ",
                };
                s.push_str(c);
            }
            s.push_str("\n");
        }
        write!(f, "{}", s)
    }
}

mod grid_line {
    use cgmath::InnerSpace;

    use super::V2D;

    pub struct GridLinePath {
        start: V2D,
        end: V2D,
        step: f64,
        current_length: f64,
    }

    impl GridLinePath {
        pub fn new(start: V2D, end: V2D, step: f64) -> Self {
            Self {
                start,
                end,
                step,
                current_length: 0.0,
            }
        }

        pub fn next(&mut self) -> Option<V2D> {
            let distance = self.start - self.end;
            if self.current_length > distance.magnitude() {
                return None;
            }
            let direction = self.end - self.start;
            let direction = direction.normalize();
            let next = self.start + direction * self.current_length;
            self.current_length += self.step;
            return Some(next);
        }
    }

    #[cfg(test)]
    mod test {
        use cgmath::InnerSpace;

        use crate::game_map::{grid_line::GridLinePath, V2D};

        #[test]
        fn test_line() {
            let mut line = GridLinePath::new(V2D::new(0.0, 0.0), V2D::new(2.0, 2.0), 1.0);
            assert_eq!(line.next(), Some(V2D::new(0.0, 0.0)));
            let point = line.next().unwrap();
            assert_eq!(point.magnitude() - 1.0 < 0.001, true);
            let point = line.next().unwrap();
            assert_eq!(point.magnitude() - 2.0 < 0.001, true);
            assert_eq!(line.next(), None);
        }
    }
}

const MAX_SEARCH: usize = 5_000;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_grid_isomorphism() {
        let grid = WorldGrid::new(40.0, Tile::default(), 10.0);
        assert_eq!(grid.data.len(), 16);
        assert_eq!(grid.tile_unit(0.0), 2);
        assert_eq!(grid.tile_unit(1.0), 2);
        assert_eq!(grid.tile_unit(9.9), 2);
        assert_eq!(grid.tile_unit(11.0), 3);
        assert_eq!(grid.tile_unit(-1.0), 1);

        //going back
        assert_eq!(grid.from_tile_unit(2), 0.0);
        assert_eq!(grid.from_tile_unit(3), 10.0);
    }

    #[test]
    fn test_pathfinding_diagonal() {
        let grid = WorldGrid::new(80.0, Tile::default(), 10.0);
        let path = grid.find_path(Vector2::new(1.0, 0.0), Vector2::new(30.0, 30.0));
        println!("{:?}", path);
        // assert_eq!(path.unwrap().len(), 4);
    }

    #[test]
    fn test_pathfinding_curve() {
        let mut grid = WorldGrid::new(80.0, Tile::default(), 10.0);
        grid.set(20.0, 20.0, Tile::grass(10.0));
        grid.calc_path_cache();
        let path = grid.find_path(Vector2::new(0.0, 0.0), Vector2::new(30.0, 30.0));
        // assert_eq!(path.unwrap().len(), 5);
        println!("{:?}", path);
    }

    #[test]
    fn test_pathfinding_impossible() {
        let grid = WorldGrid::new(80.0, Tile::default(), 10.0);
        let path = grid.find_path(Vector2::new(0.0, 0.0), Vector2::new(300.0, 30.0));
        assert_eq!(path, None);
    }

    #[test]
    fn flood_fill() {
        let mut grid = WorldGrid::new(4.0, Tile::default(), 1.0);

        grid.set(0.0, 0.0, Tile::grass(1.0));
        grid.set(0.0, 1.0, Tile::grass(1.0));
        grid.set(1.0, 1.0, Tile::grass(1.0));
        grid.set(1.0, 0.0, Tile::grass(1.0));
        println!("{:?}", grid);

        grid.find_islands();
        println!("{:?}", grid.islands);
    }
}
