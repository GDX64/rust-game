use std::collections::HashMap;

use crate::game_noise::NoiseConfig;

use super::game_noise::GameNoise;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WorldGen {
    low_land: GameNoise,
    high_land: GameNoise,
    forest: GameNoise,
    config: WorldGenConfig,
    tiles: SparseMatrix<TileKind>,
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
        }
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

    pub fn get_rect(&mut self, x: f64, y: f64, rows: usize, cols: usize) -> Vec<TileKind> {
        let tile_size = self.config.tile_size.unwrap_or(1.0);
        let mut tiles = Vec::with_capacity(rows * cols);
        for i in 0..rows {
            for j in 0..cols {
                let tile = self.get_terrain_at(x + j as f64 * tile_size, y + i as f64 * tile_size);
                tiles.push(tile);
            }
        }
        tiles
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
