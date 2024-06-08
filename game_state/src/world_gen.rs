use std::collections::HashMap;

use crate::game_noise::NoiseConfig;

use super::game_noise::GameNoise;
use cgmath::{EuclideanSpace, Matrix3, Point2, SquareMatrix, Transform};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WorldGen {
    low_land: GameNoise,
    high_land: GameNoise,
    forest: GameNoise,
    config: WorldGenConfig,
    tiles: SparseMatrix<TileKind>,
    matrix: Matrix3<f64>,
}

#[wasm_bindgen]
#[derive(Clone, Copy, Default)]
pub struct ViewInfo {
    pub x_center: f64,
    pub y_center: f64,
    pub pixels: f64,
    pub range: f64,
}

#[wasm_bindgen]
impl ViewInfo {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ViewInfo {
    pub fn to_matrix(&self) -> Matrix3<f64> {
        lin_scale(
            Point2::new(0.0, 0.0),
            Point2::new(self.pixels, self.pixels),
            Point2::new(self.x_center - self.range, self.y_center - self.range),
            Point2::new(self.x_center + self.range, self.y_center + self.range),
        )
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy, Default)]
pub struct WorldGenConfig {
    pub low_land: Option<NoiseConfig>,
    pub high_land: Option<NoiseConfig>,
    pub forest: Option<NoiseConfig>,
    pub weight_low_land: Option<f64>,
    pub forest_threshold: Option<f64>,
    pub land_threshold: Option<f64>,
    pub tile_size: Option<f64>,
    pub view_info: Option<ViewInfo>,
}

#[wasm_bindgen]
impl WorldGenConfig {
    pub fn new() -> Self {
        Self::default()
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub enum TileKind {
    Water,
    Grass,
    Forest,
}

#[wasm_bindgen]
impl WorldGen {
    pub fn new(seed: u32) -> Self {
        Self {
            low_land: GameNoise::new(Some(seed)),
            high_land: GameNoise::new(Some(seed)),
            forest: GameNoise::new(Some(seed)),
            config: WorldGenConfig::default(),
            tiles: SparseMatrix::new(),
            matrix: Matrix3::identity(),
        }
    }

    pub fn transform_point(&self, x: f64, y: f64) -> Vec<f64> {
        let vec = self.matrix.transform_point(Point2::new(x, y));
        vec![vec.x, vec.y]
    }

    pub fn set_config(&mut self, config: WorldGenConfig) {
        if let Some(low_land) = config.low_land {
            self.low_land.set_config(low_land);
        }
        if let Some(high_land) = config.high_land {
            self.high_land.set_config(high_land);
        }
        if let Some(forest) = config.forest {
            self.forest.set_config(forest);
        }
        self.config = config;
        if let Some(view_info) = config.view_info {
            self.matrix = view_info.to_matrix();
        }
        self.tiles.clear();
    }

    pub fn get_tile_at(&mut self, x: f64, y: f64) -> TileKind {
        let tile_size = self.config.tile_size.unwrap_or(1.0);
        let x = (x / tile_size).floor();
        let y = (y / tile_size).floor();
        let cached = self.tiles.get(x as i32, y as i32);
        if let Some(tile) = cached {
            return *tile;
        } else {
            let tile = self.get_terrain_at(x * tile_size, y * tile_size);
            self.tiles.set(x as i32, y as i32, tile);
            return tile;
        }
    }

    pub fn get_canvas(&mut self) -> Option<Vec<TileKind>> {
        let pixels = self.config.view_info?.pixels;
        let mut tiles = Vec::with_capacity(pixels as usize * pixels as usize);
        for y in 0..pixels as i32 {
            for x in 0..pixels as i32 {
                let point = self.matrix.transform_point(Point2::new(x as f64, y as f64));
                tiles.push(self.get_terrain_at(point.x, point.y));
            }
        }
        Some(tiles)
    }

    pub fn get_terrain_at(&self, x: f64, y: f64) -> TileKind {
        let low_land = self.low_land.get(x, y);
        let high_land = self.high_land.get(x, y);
        let low_land_weight = self.config.weight_low_land.unwrap_or(0.5);
        let land_value = low_land_weight * low_land + (1.0 - low_land_weight) * high_land;

        let terrain_kind = if land_value < self.config.land_threshold.unwrap_or(0.3) {
            TileKind::Water
        } else {
            if self.forest.get(x, y) > self.config.forest_threshold.unwrap_or(0.5) {
                TileKind::Forest
            } else {
                TileKind::Grass
            }
        };

        terrain_kind
    }
}

struct SparseMatrix<T> {
    data: HashMap<(i32, i32), T>,
}

impl<T> SparseMatrix<T> {
    fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    fn get(&self, x: i32, y: i32) -> Option<&T> {
        self.data.get(&(x, y))
    }

    fn set(&mut self, x: i32, y: i32, value: T) {
        self.data.insert((x, y), value);
    }

    fn clear(&mut self) {
        self.data.clear();
    }
}

#[cfg(test)]
mod test {
    use cgmath::Transform;

    use crate::world_gen::lin_scale;

    #[test]
    fn test() {
        let camera = lin_scale(
            cgmath::Point2::new(0.0, 0.0),
            cgmath::Point2::new(100.0, 100.0),
            cgmath::Point2::new(-1.0, -1.0),
            cgmath::Point2::new(1.0, 1.0),
        );

        let v = camera.transform_point(cgmath::Point2::new(100.0, 100.0));
        assert_eq!(v, cgmath::Point2::new(1.0, 1.0));

        let v = camera.transform_point(cgmath::Point2::new(50.0, 50.0));
        assert_eq!(v, cgmath::Point2::new(0.0, 0.0));

        let v = camera.transform_point(cgmath::Point2::new(0.0, 0.0));
        assert_eq!(v, cgmath::Point2::new(-1.0, -1.0));
    }
}

fn lin_scale(x0: Point2<f64>, x1: Point2<f64>, y0: Point2<f64>, y1: Point2<f64>) -> Matrix3<f64> {
    let delta_x = x1 - x0;
    let delta_y = y1 - y0;
    let scale_x = delta_y.x / delta_x.x;
    let scale_y = delta_y.y / delta_x.y;
    let alpha = Matrix3::from_nonuniform_scale(scale_x, scale_y);
    let beta = y0 - alpha.transform_point(x0);
    let beta_matrix = Matrix3::from_translation(beta);
    return beta_matrix * alpha;
}
