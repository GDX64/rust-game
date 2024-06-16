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

pub struct TileUnit(usize);

impl TileUnit {
    pub fn div(val: f64, tile_size: f64) -> Self {
        Self((val / tile_size).floor() as usize)
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

    fn tile_unit(&self, val: f64) -> usize {
        TileUnit::div(val + self.dim / 2.0, self.tile_size).0
    }

    fn from_tile_unit(&self, unit: usize) -> f64 {
        (unit as f64 * self.tile_size) - self.dim / 2.0
    }

    pub fn get(&self, x: f64, y: f64) -> Option<&T> {
        let x = self.tile_unit(x);
        let y = self.tile_unit(y);
        let index = (y * self.tiles_dim + x) as usize;
        self.data.get(index)
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
}
