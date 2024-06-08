use crate::game_noise::NoiseConfig;

use super::game_noise::GameNoise;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct WorldGen {
    low_land: GameNoise,
    high_land: GameNoise,
    forest: GameNoise,
    config: WorldGenConfig,
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
    }

    pub fn get_tile_at(&self, x: f64, y: f64) -> TileKind {
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
