use std::vec;

use cgmath::{InnerSpace, Vector2, Vector3};
use pathfinding::prelude::astar;

pub struct WorldGrid<T> {
    pub dim: f64,
    pub tiles_dim: usize,
    pub tile_size: f64,
    pub data: Vec<T>,
}

impl<T> Default for WorldGrid<T> {
    fn default() -> Self {
        Self {
            dim: 0.0,
            tiles_dim: 0,
            tile_size: 1.0,
            data: Vec::new(),
        }
    }
}

pub trait CanGo {
    fn can_go(&self) -> bool;
}

pub struct TileUnit(usize);

impl TileUnit {
    pub fn div(val: f64, tile_size: f64) -> Self {
        Self((val / tile_size).floor() as usize)
    }
}

impl<T> WorldGrid<T> {
    fn tile_unit(&self, val: f64) -> usize {
        TileUnit::div(val + self.dim / 2.0, self.tile_size).0
    }

    fn from_tile_unit(&self, unit: usize) -> f64 {
        (unit as f64 * self.tile_size) - self.dim / 2.0
    }

    pub fn get(&self, x: f64, y: f64) -> Option<&T> {
        let x = self.tile_unit(x);
        let y = self.tile_unit(y);
        return self.get_tiles(x, y);
    }

    fn get_tiles(&self, x: usize, y: usize) -> Option<&T> {
        let index = (y * self.tiles_dim + x) as usize;
        self.data.get(index)
    }
}

impl<T: Copy + Clone> WorldGrid<T> {
    pub fn new(dim: f64, default: T, tile_size: f64) -> Self {
        let tiles_dim = TileUnit::div(dim, tile_size).0;

        Self {
            dim,
            tiles_dim,
            tile_size,
            data: vec![default; tiles_dim * tiles_dim],
        }
    }

    pub fn iter(&mut self) -> impl Iterator<Item = (f64, f64, &T)> {
        self.data.iter().enumerate().map(|(index, value)| {
            let x = index % self.tiles_dim;
            let y = index / self.tiles_dim;
            let x = self.from_tile_unit(x);
            let y = self.from_tile_unit(y);
            (x, y, value)
        })
    }

    pub fn get_mut(&mut self, x: f64, y: f64) -> Option<&mut T> {
        let x = self.tile_unit(x);
        let y = self.tile_unit(y);
        let index = (y * self.tiles_dim + x) as usize;
        self.data.get_mut(index)
    }

    pub fn set(&mut self, x: f64, y: f64, value: T) {
        let x = self.tile_unit(x);
        let y = self.tile_unit(y);
        let index = (y * self.tiles_dim + x) as usize;
        self.data[index] = value;
    }
}

pub type V2D = Vector2<f64>;
pub type V3D = Vector3<f64>;

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

        use crate::sparse_matrix::{grid_line::GridLinePath, V2D};

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

const MAX_SEARCH: usize = 10_000;

impl<T: CanGo> WorldGrid<T> {
    pub fn is_allowed_place(&self, x: f64, y: f64) -> bool {
        let x = self.tile_unit(x);
        let y = self.tile_unit(y);
        if let Some(value) = self.get_tiles(x, y) {
            return value.can_go();
        }
        return false;
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

    pub fn find_path(&self, initial: impl Into<V2D>, fin: impl Into<V2D>) -> Option<Vec<V2D>> {
        let initial = initial.into();
        let fin = fin.into();
        if self.can_go_straight(&initial, &fin) {
            return Some(vec![initial.into(), fin.into()]);
        }
        let initial = Vector2::new(
            self.tile_unit(initial.x) as i64,
            self.tile_unit(initial.y) as i64,
        );
        let mut i = 0;
        let fin = Vector2::new(self.tile_unit(fin.x) as i64, self.tile_unit(fin.y) as i64);
        let goal_fn = |p: &Vector2<i64>| (fin - p).magnitude2();
        let result = astar(
            &initial,
            |p| {
                let get_info = |x: i64, y: i64| {
                    if x < 0 || y < 0 {
                        return None;
                    }
                    let value = self.get_tiles(x as usize, y as usize)?;
                    if value.can_go() {
                        let point = Vector2::new(x, y);
                        return Some((point, goal_fn(&point)));
                    }
                    return None;
                };
                let nw = get_info(p.x - 1, p.y - 1);
                let n = get_info(p.x, p.y - 1);
                let ne = get_info(p.x + 1, p.y - 1);
                let w = get_info(p.x - 1, p.y);
                let e = get_info(p.x + 1, p.y);
                let sw = get_info(p.x - 1, p.y + 1);
                let s = get_info(p.x, p.y + 1);
                let se = get_info(p.x + 1, p.y + 1);
                return vec![nw, n, ne, w, e, sw, s, se]
                    .into_iter()
                    .filter_map(|x| x);
            },
            goal_fn,
            |p| {
                i += 1;
                return *p == fin || i > MAX_SEARCH;
            },
        )?;
        if i > MAX_SEARCH {
            return None;
        }
        let v: Vec<Vector2<f64>> = result
            .0
            .into_iter()
            .map(|v| {
                Vector2::new(
                    self.from_tile_unit(v.x as usize),
                    self.from_tile_unit(v.y as usize),
                )
            })
            .collect();
        return Some(v);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_grid_isomorphism() {
        let grid = WorldGrid::new(40.0, 0.0, 10.0);
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

    impl CanGo for bool {
        fn can_go(&self) -> bool {
            return *self;
        }
    }

    #[test]
    fn test_pathfinding_diagonal() {
        let grid = WorldGrid::new(80.0, true, 10.0);
        let path = grid.find_path(Vector2::new(1.0, 0.0), Vector2::new(30.0, 30.0));
        assert_eq!(path.unwrap().len(), 4);
    }

    #[test]
    fn test_pathfinding_curve() {
        let mut grid = WorldGrid::new(80.0, true, 10.0);
        grid.set(20.0, 20.0, false);
        let path = grid.find_path(Vector2::new(0.0, 0.0), Vector2::new(30.0, 30.0));
        assert_eq!(path.unwrap().len(), 5);
    }

    #[test]
    fn test_pathfinding_impossible() {
        let grid = WorldGrid::new(80.0, true, 10.0);
        let path = grid.find_path(Vector2::new(0.0, 0.0), Vector2::new(300.0, 30.0));
        assert_eq!(path, None);
    }
}
