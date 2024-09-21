use crate::{game_map::V2D, spiral_search::moore_neighborhood};
use cgmath::InnerSpace;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashSet};

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
        let grid_width = bounds.2 - bounds.0 + 1;
        let grid_height = bounds.3 - bounds.1 + 1;
        let grid_width = grid_width as usize;
        let grid_height = grid_height as usize;
        let mut land_grid = vec![vec![false; grid_width]; grid_height];
        for tile in self.tiles.iter() {
            land_grid[(tile.y - bounds.1) as usize][(tile.x - bounds.0) as usize] = true;
        }
        let mut coast_grid = vec![vec![false; grid_width]; grid_height];

        //scan left side
        for y in 0..grid_height {
            for x in 0..grid_width {
                if land_grid[y][x] {
                    coast_grid[y][x] = true;
                    break;
                }
            }
        }

        //scan bottom side
        for x in 0..grid_width {
            for y in (0..grid_height).rev() {
                if land_grid[y][x] {
                    coast_grid[y][x] = true;
                    break;
                }
            }
        }

        for y in (0..grid_height).rev() {
            for x in (0..grid_width).rev() {
                if land_grid[y][x] {
                    coast_grid[y][x] = true;
                    break;
                }
            }
        }

        for x in (0..grid_width).rev() {
            for y in 0..grid_height {
                if land_grid[y][x] {
                    coast_grid[y][x] = true;
                    break;
                }
            }
        }

        let mut x = 0i32;
        let mut y = 0i32;
        for cx in 0..grid_width {
            if coast_grid[0][cx] {
                x = cx as i32;
                break;
            }
        }
        //now we walk the coast in a clockwise direction
        let mut history = Vec::new();
        let mut border = Vec::new();
        border.push((x, y));
        'outer: loop {
            for (nx, ny) in moore_neighborhood(x as i32, y as i32).into_iter() {
                if nx < 0 || ny < 0 {
                    continue;
                }
                let is_coast = *coast_grid
                    .get(ny as usize)
                    .and_then(|v| v.get(nx as usize))
                    .unwrap_or(&false);
                if is_coast && !border.contains(&(nx, ny)) {
                    history.push((x, y));
                    x = nx;
                    y = ny;
                    border.push((x, y));
                    continue 'outer;
                } else {
                    continue;
                }
            }
            match history.pop() {
                Some((hx, hy)) => {
                    x = hx;
                    y = hy;
                }
                None => {
                    break;
                }
            }
        }

        let half_width = (grid_width as f64) * self.tile_size / 2.0;
        let half_height = (grid_height as f64) * self.tile_size / 2.0;

        let border: Vec<_> = border
            .into_iter()
            .map(|(x, y)| {
                let x = (x as f64) * self.tile_size;
                let y = (y as f64) * self.tile_size;
                return (
                    x + self.center.x - half_width,
                    y + self.center.y - half_height,
                );
            })
            .collect();
        let border = douglas_peucker(&border, error);
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

fn douglas_peucker(points: &[(f64, f64)], epsilon: f64) -> Vec<(f64, f64)> {
    let mut dmax = 0.0;
    let mut index = 0;
    let end = points.len() - 1;
    for i in 1..end {
        let d = perpendicular_distance(&points[i], &points[0], &points[end]);
        if d > dmax {
            index = i;
            dmax = d;
        }
    }
    if dmax > epsilon {
        let mut res1 = douglas_peucker(&points[..=index], epsilon);
        let res2 = douglas_peucker(&points[index..], epsilon);
        res1.pop();
        res1.extend(res2);
        return res1;
    } else {
        return vec![points[0], points[end]];
    }
}

fn perpendicular_distance(point: &(f64, f64), start: &(f64, f64), end: &(f64, f64)) -> f64 {
    let point = V2D::new(point.0, point.1);
    let start = V2D::new(start.0, start.1);
    let end = V2D::new(end.0, end.1);
    let line = end - start;
    let len = line.magnitude();
    let line = line / len;
    let point = point - start;
    let projection = line.dot(point);
    let projection = projection.max(0.0).min(len);
    let projection = start + line * projection;
    let distance = point - projection;
    distance.magnitude()
}
