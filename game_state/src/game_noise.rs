use noise::{Fbm, MultiFractal, NoiseFn, Seedable, Simplex};
use wasm_bindgen::prelude::*;

type NoiseKind = Simplex;

#[wasm_bindgen]
#[derive(Clone)]
pub struct GameNoise {
    fbm1: Fbm<NoiseKind>,
}

#[wasm_bindgen]
#[derive(Clone, Copy, Default)]
pub struct NoiseConfig {
    pub seed: Option<u32>,
    pub frequency: Option<f64>,
    pub octaves: Option<u32>,
    pub lacunarity: Option<f64>,
    pub persistence: Option<f64>,
}

#[wasm_bindgen]
impl NoiseConfig {
    pub fn new() -> Self {
        Self::default()
    }
}

#[wasm_bindgen]
impl GameNoise {
    pub fn new(seed: Option<u32>) -> Self {
        let fbm1 = Fbm::<NoiseKind>::new(seed.unwrap_or(0));
        Self { fbm1: fbm1.clone() }
    }

    pub fn set_config(&mut self, config: NoiseConfig) {
        if let Some(seed) = config.seed {
            self.fbm1 = self.fbm1.clone().set_seed(seed);
        }
        if let Some(frequency) = config.frequency {
            self.fbm1 = self.fbm1.clone().set_frequency(frequency);
        }
        if let Some(octaves) = config.octaves {
            self.fbm1 = self.fbm1.clone().set_octaves(octaves as usize);
        }
        if let Some(lacunarity) = config.lacunarity {
            self.fbm1 = self.fbm1.clone().set_lacunarity(lacunarity);
        }
        if let Some(persistence) = config.persistence {
            self.fbm1 = self.fbm1.clone().set_persistence(persistence);
        }
    }

    pub fn set_persistence(&mut self, persistence: f64) {
        self.fbm1 = self.fbm1.clone().set_persistence(persistence);
    }

    pub fn set_frequency(&mut self, frequency: f64) {
        self.fbm1 = self.fbm1.clone().set_frequency(frequency);
    }

    pub fn set_seed(&mut self, seed: u32) {
        self.fbm1 = self.fbm1.clone().set_seed(seed);
    }

    pub fn set_octaves(&mut self, octaves: usize) {
        self.fbm1 = self.fbm1.clone().set_octaves(octaves);
    }

    pub fn set_lacunarity(&mut self, lacunarity: f64) {
        self.fbm1 = self.fbm1.clone().set_lacunarity(lacunarity);
    }

    pub fn get(&self, x: f64, y: f64) -> f64 {
        self.fbm1.get([x, y])
    }

    pub fn generate(&self, size: usize) -> Vec<u8> {
        let v: Vec<u8> = (0..size)
            .flat_map(|row| {
                (0..size).flat_map(move |col| {
                    let value = self.fbm1.get([col as f64, row as f64]);
                    let value = self.step(value);
                    let value = (value + 1.0) * 255.0 / 2.0;
                    let color = (value) as u8;
                    return [color, color, color, 255];
                })
            })
            .collect();
        return v;
    }

    fn step(&self, value: f64) -> f64 {
        if value > 0.5 {
            return 1.0;
        } else {
            return 0.0;
        }
    }
}
