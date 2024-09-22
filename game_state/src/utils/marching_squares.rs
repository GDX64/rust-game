use marching_squares::{Field, Line, Point};
use std::{
    fmt::{Debug, Formatter},
    ops::BitOrAssign,
};

trait GridMust: Clone + Copy + Default {}

impl<T: Clone + Copy + Default> GridMust for T {}

struct Grid<T> {
    size: usize,
    grid: Vec<T>,
}

struct FloatGrid<T> {
    size: f64,
    tile_size: f64,
    grid: Grid<T>,
}

impl<T: GridMust> FloatGrid<T> {
    pub fn new(size: f64, tile_size: f64, default: T) -> Self {
        let grid_size = (size / tile_size).ceil() as usize;
        Self {
            tile_size,
            size,
            grid: Grid::new(grid_size, default),
        }
    }

    fn to_tile(&self, x: f64) -> usize {
        ((x + self.size / 2.0) / self.tile_size) as usize
    }

    fn from_tile(&self, x: usize) -> f64 {
        x as f64 * self.tile_size - self.size / 2.0
    }

    pub fn get_or_default(&self, x: f64, y: f64) -> T {
        let x = self.to_tile(x);
        let y = self.to_tile(y);
        self.grid.get_or_default(x, y)
    }

    pub fn indexes(&self) -> impl Iterator<Item = (f64, f64)> {
        let tile_size = self.tile_size;
        let size = self.size;
        let from_tile = move |x: usize| {
            return x as f64 * tile_size - size / 2.0;
        };
        let size = self.grid.size;
        return (0..size).flat_map(move |x| (0..size).map(move |y| (from_tile(x), from_tile(y))));
    }

    pub fn set(&mut self, x: f64, y: f64, value: T) {
        let x = self.to_tile(x);
        let y = self.to_tile(y);
        self.grid.set(x, y, value);
    }
}

impl<T: GridMust> Grid<T> {
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

    pub fn map<U: GridMust, F: Fn(T) -> U>(&self, f: F) -> Grid<U> {
        let grid = self.grid.iter().map(|v| f(*v)).collect();
        Grid {
            size: self.size,
            grid,
        }
    }

    pub fn to_vec_vec(&self) -> Vec<Vec<T>> {
        let size = self.size;
        (0..size)
            .map(|y| (0..size).map(|x| self.get(x, y).unwrap()).collect())
            .collect()
    }
}

impl<T: GridMust + Debug> Debug for Grid<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut string = String::new();
        self.grid.iter().enumerate().for_each(|(i, v)| {
            string.push_str(&format!("{:?}", v));
            if i % self.size == 0 && i != 0 {
                string.push_str("\n");
            }
        });
        write!(f, "{} ", string)
    }
}

fn march_on_bool_grid<T: Into<i16> + GridMust>(
    grid: &Grid<T>,
    pixel_size: f32,
    top_x: f32,
    top_y: f32,
) -> Vec<Line> {
    let veci16 = grid.map(|v| return v.into()).to_vec_vec();

    let field = Field {
        dimensions: (grid.size, grid.size),
        pixel_size: (pixel_size, pixel_size),
        top_left: Point { x: top_x, y: top_y },
        values: &veci16,
    };

    let lines = field.get_contours(0);
    return lines;
}

#[cfg(test)]
mod test {
    use tiny_skia::PathBuilder;

    use crate::utils::marching_squares::{march_on_bool_grid, FloatGrid};

    const CIRCLE_RADIUS: f64 = 40.0;
    #[test]
    fn test_circle_function() {
        fn is_in_circle(x: f64, y: f64) -> bool {
            x * x + y * y < CIRCLE_RADIUS * CIRCLE_RADIUS
        }

        let size = 100;

        let mut grid = FloatGrid::new(size as f64, 1.0, 0u8);
        grid.indexes()
            .for_each(|(x, y)| grid.set(x, y, is_in_circle(x, y) as u8));

        let marched = march_on_bool_grid(&grid.grid, 1.0, 0.0, 0.0);

        let mut skia_plot = tiny_skia::Pixmap::new(size, size).unwrap();
        for line in marched {
            let mut path = PathBuilder::new();
            let mut line = line.points.into_iter();
            let first = line.next().unwrap();
            path.move_to(first.x as f32, first.y as f32);
            for point in line {
                path.line_to(point.x as f32, point.y as f32);
            }
            skia_plot.stroke_path(
                &path.finish().unwrap(),
                &tiny_skia::Paint::default(),
                &tiny_skia::Stroke::default(),
                tiny_skia::Transform::identity(),
                None,
            );
        }
        skia_plot.save_png("./example_images/marched.png").unwrap();
    }
}
