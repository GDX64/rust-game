use crate::utils::{
    marching_squares::{march_on_grid, Grid},
    spiral_search::manhattan_neighborhood,
    vectors::V2D,
};
use cgmath::InnerSpace;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct IslandData {
    pub id: u64,
    pub center: (f64, f64),
    pub light_house: (f64, f64),
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct IslandTile {
    x: i32,
    y: i32,
    height: f64,
}

impl IslandTile {
    pub fn new(height: f64, x: i32, y: i32) -> Self {
        Self { x, y, height }
    }
}

impl Eq for IslandTile {}

impl Ord for IslandTile {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.x, self.y).partial_cmp(&(other.x, other.y)).unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct Island {
    pub tiles: BTreeSet<IslandTile>,
    pub id: u64,
    pub center: V2D,
    pub light_house: V2D,
    pub tile_size: f64,
}

impl Island {
    fn bounding_box(&self) -> (i32, i32, i32, i32) {
        let mut min_x = i32::MAX;
        let mut min_y = i32::MAX;
        let mut max_x = i32::MIN;
        let mut max_y = i32::MIN;
        for tile in self.tiles.iter() {
            min_x = min_x.min(tile.x);
            min_y = min_y.min(tile.y);
            max_x = max_x.max(tile.x);
            max_y = max_y.max(tile.y);
        }
        (min_x, min_y, max_x, max_y)
    }

    pub fn new(tiles: BTreeSet<IslandTile>, number: u64, tile_size: f64) -> Self {
        Self {
            tiles,
            id: number,
            center: (0.0, 0.0).into(),
            light_house: (0.0, 0.0).into(),
            tile_size,
        }
    }

    pub fn calc_center(&mut self) -> V2D {
        let mut x = 0.0;
        let mut y = 0.0;
        for tile in self.tiles.iter() {
            x += tile.x as f64;
            y += tile.y as f64;
        }
        x /= self.tiles.len() as f64;
        y /= self.tiles.len() as f64;
        let center = V2D::new(x, y);
        center
    }

    pub fn island_path(&self, error: f64) -> Vec<(f64, f64)> {
        let bounds = self.bounding_box();
        let padding = 2;
        let grid_width = bounds.2 - bounds.0 + 1 + padding * 2;
        let grid_height = bounds.3 - bounds.1 + 1 + padding * 2;
        let size = grid_width.max(grid_height);
        let grid_width = grid_width as usize;
        let grid_height = grid_height as usize;
        let mut land_grid = Grid::new(size as usize, 0i16);
        let mut sea_grid = Grid::new(size as usize, 0i16);

        for tile in self.tiles.iter() {
            let y = tile.y - bounds.1;
            let x = tile.x - bounds.0;
            land_grid.set((x + padding) as usize, (y + padding) as usize, 1);
        }

        //flood fill the sea
        // let mut stack = BTreeSet::new();
        // stack.insert((0, 0));
        // while let Some((x, y)) = stack.pop_first() {
        //     if x >= grid_width || y >= grid_height {
        //         continue;
        //     }
        //     if sea_grid.get_or_default(x, y) == 1 {
        //         continue;
        //     }
        //     if land_grid.get_or_default(x, y) == 1 {
        //         continue;
        //     }
        //     sea_grid.set(x, y, 1);
        //     stack.insert((x + 1, y));
        //     stack.insert((x.checked_sub(1).unwrap_or(0), y));
        //     stack.insert((x, y + 1));
        //     stack.insert((x, y.checked_sub(1).unwrap_or(0)));
        // }

        log::info!("{:?}", land_grid);

        let half_width = (grid_width as f64) * self.tile_size / 2.0;
        let half_height = (grid_height as f64) * self.tile_size / 2.0;

        let lines = march_on_grid(
            &land_grid,
            self.tile_size as f32,
            (self.center.x - half_width) as f32,
            (self.center.y - half_height) as f32,
        );

        let border: Vec<_> = lines
            .into_iter()
            .max_by_key(|line| line.points.len())
            .into_iter()
            .flat_map(|points| points.points)
            .map(|point| {
                return (point.x as f64, point.y as f64);
            })
            .collect();
        return border;
    }

    pub fn island_data(&self) -> IslandData {
        IslandData {
            id: self.id,
            center: (self.center.x, self.center.y),
            light_house: (self.light_house.x, self.light_house.y),
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::BTreeSet;

    use super::IslandTile;

    fn make_a_grid<const N: usize>(grid: [[u8; N]; N]) -> BTreeSet<IslandTile> {
        let mut tiles = BTreeSet::new();
        for (i, line) in grid.iter().enumerate() {
            for (j, c) in line.iter().enumerate() {
                if *c == 1 {
                    tiles.insert(IslandTile::new(0.0, i as i32, j as i32));
                }
            }
        }
        return tiles;
    }

    #[test]
    fn test() {
        let tiles = make_a_grid([
            //0  1  2
            [0, 1, 0, 0], // 0
            [1, 1, 1, 1], // 1
            [0, 1, 0, 0], // 2
            [0, 1, 0, 0], // 2
        ]);
        let island = super::Island::new(tiles, 0, 1.0);
        let path = island.island_path(0.1);
        println!("{:?}", path);
    }
}
