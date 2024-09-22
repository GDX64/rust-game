use std::ops::BitOrAssign;

struct Grid<T> {
    size: usize,
    grid: Vec<T>,
}

impl<T: Clone + Copy + Default> Grid<T> {
    pub fn new(size: usize, default: T) -> Self {
        Self {
            size,
            grid: vec![default; size * size],
        }
    }

    pub fn index_of(&self, x: usize, y: usize) -> usize {
        x + y * self.size
    }

    pub fn get(&self, x: usize, y: usize) -> Option<T> {
        self.grid.get(self.index_of(x, y)).cloned()
    }

    pub fn get_or_default(&self, x: usize, y: usize) -> T {
        self.get(x, y).unwrap_or_default()
    }

    pub fn set(&mut self, x: usize, y: usize, value: T) {
        let index = self.index_of(x, y);
        if let Some(r) = self.grid.get_mut(index) {
            *r = value;
        }
    }
}

pub fn march_on_bool_grid<T: Into<u8> + Clone + Copy + Default>(grid: &Grid<T>) {
    let n = grid.size;
    if n < 2 {
        return;
    }
    let mut march_grid = Grid::new(n - 1, 0u8);
    for y in 0..march_grid.size {
        for x in 0..march_grid.size {
            let nw = grid.get_or_default(x, y).into();
            let ne = grid.get_or_default(x + 1, y).into();
            let se = grid.get_or_default(x + 1, y + 1).into();
            let sw = grid.get_or_default(x, y + 1).into();
            let result = (ne << 3) | (nw << 2) | (se << 1) | sw;
            march_grid.set(x, y, result);
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum MarchTile {}
