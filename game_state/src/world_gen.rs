use super::game_noise::GameNoise;
use crate::{
    game_map::{Tile, TileKind, WorldGrid},
    game_noise::NoiseConfig,
    interpolation::LinearInterpolation,
};
use cgmath::{Matrix3, Point2, SquareMatrix, Transform};
use log::info;
use wasm_bindgen::prelude::*;
#[wasm_bindgen]
#[derive(Clone)]
pub struct WorldGen {
    low_land: GameNoise,
    high_land: GameNoise,
    forest: GameNoise,
    config: WorldGenConfig,
    matrix: Matrix3<f64>,
    terrain_interpolation: LinearInterpolation,
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct ViewInfo {
    pub x_center: f64,
    pub y_center: f64,
    pub pixels: f64,
    pub range: f64,
}

impl Default for ViewInfo {
    fn default() -> Self {
        Self {
            x_center: 0.0,
            y_center: 0.0,
            pixels: 100.0,
            range: 1.0,
        }
    }
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
#[derive(Clone, Copy)]
pub struct WorldGenConfig {
    pub low_land: Option<NoiseConfig>,
    pub high_land: Option<NoiseConfig>,
    pub forest: Option<NoiseConfig>,
    pub height_scale: f64,
    pub weight_low_land: f64,
    pub forest_threshold: f64,
    pub land_threshold: f64,
    pub tile_size: f64,
    pub noise_scale: f64,
    pub view_info: ViewInfo,
}

impl Default for WorldGenConfig {
    fn default() -> Self {
        let low_land = NoiseConfig {
            frequency: Some(1.0),
            ..NoiseConfig::default()
        };
        let high_land = NoiseConfig {
            frequency: Some(23.0),
            ..NoiseConfig::default()
        };
        let forest = NoiseConfig {
            frequency: Some(5.0),
            ..NoiseConfig::default()
        };
        Self {
            low_land: Some(low_land),
            high_land: Some(high_land),
            forest: Some(forest),
            weight_low_land: 0.9,
            forest_threshold: 0.17,
            land_threshold: 0.0,
            noise_scale: 0.001,
            tile_size: 20.0,
            height_scale: 500.0,
            view_info: ViewInfo::default(),
        }
    }
}

#[wasm_bindgen]
impl WorldGenConfig {
    pub fn new() -> Self {
        Self::default()
    }
}

#[wasm_bindgen]
impl WorldGen {
    pub fn new(seed: u32) -> Self {
        Self {
            terrain_interpolation: LinearInterpolation::new(vec![
                Point2::new(-1.0, -1.0),
                Point2::new(0.10, 0.0),
                Point2::new(0.5, 0.15),
                Point2::new(0.7, 0.4),
                Point2::new(1.0, 0.5),
            ]),
            low_land: GameNoise::new(Some(seed)),
            high_land: GameNoise::new(Some(seed)),
            forest: GameNoise::new(Some(seed)),
            config: WorldGenConfig::default(),
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
        self.matrix = config.view_info.to_matrix();
        self.config = config;
    }

    pub fn get_canvas(&mut self) -> Vec<TileKind> {
        let pixels = self.config.view_info.pixels;
        let mut tiles = Vec::with_capacity(pixels as usize * pixels as usize);
        for y in 0..pixels as i32 {
            for x in 0..pixels as i32 {
                let point = self.matrix.transform_point(Point2::new(x as f64, y as f64));
                tiles.push(self.get_terrain_at(point.x, point.y).kind());
            }
        }
        tiles
    }

    pub fn get_land_value(&self, x: f64, y: f64) -> f64 {
        let x = x * self.config.noise_scale;
        let y = y * self.config.noise_scale;
        let low_land = self.low_land.get(x, y);
        let high_land = self.high_land.get(x, y);
        let low_land_weight = self.config.weight_low_land;
        let land_value = low_land_weight * low_land + (1.0 - low_land_weight) * high_land;
        self.terrain_interpolation
            .interpolate(land_value)
            .unwrap_or(0.0)
            * self.config.height_scale
    }

    fn get_terrain_at(&self, x: f64, y: f64) -> Tile {
        let land_value = self.get_land_value(x, y);
        let terrain_kind = if land_value < self.config.land_threshold {
            TileKind::Water
        } else {
            if self.forest.get(x, y) > self.config.forest_threshold {
                TileKind::Forest
            } else {
                TileKind::Grass
            }
        };

        Tile::new(terrain_kind, land_value)
    }
}

impl WorldGen {
    pub fn generate_grid(&self, width: f64) -> WorldGrid {
        let mut grid = WorldGrid::new(width, Tile::default(), self.config.tile_size);
        let data = grid
            .iter()
            .map(|(x, y, _)| {
                return self.get_terrain_at(x, y);
            })
            .collect();
        grid.data = data;
        grid.find_islands();
        grid
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
