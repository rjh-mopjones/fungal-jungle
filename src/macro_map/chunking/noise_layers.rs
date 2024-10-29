use bevy::asset::Handle;
use bevy::prelude::Image;
use noise::{NoiseFn, OpenSimplex};

#[derive(Default)]
pub struct NoiseValues {
    continentalness: f64,
    temperature: f64,
    altitude: f64
}

#[derive(Default)]
pub struct NoiseLayers {
    continentalness: Handle<Image>,
    temperature: Handle<Image>,
    altitude: Handle<Image>
}

pub struct NoiseStrategies {
    pub continentalness_strategy: ContinentalnessStrategy,
    pub temperature_strategy: TemperatureStrategy,
    pub altitude_strategy: AltitudeStrategy
}

impl NoiseStrategies {
    pub fn generate(&self, x: f64, y: f64, detail_level: u32) -> NoiseValues {
        NoiseValues {
            continentalness: self.continentalness_strategy.generate(x, y, detail_level),
            temperature: self.temperature_strategy.generate(x, y, detail_level),
            altitude: self.altitude_strategy.generate(x, y, detail_level),
        }
    }
}
pub struct AltitudeStrategy {
    scale: f64,
    mountain_scale: f64,
    noise: OpenSimplex
}

impl AltitudeStrategy {
    pub fn new(seed: u32) -> Self {
        Self {
            scale: 200.0,
            mountain_scale: 50.0,
            noise: OpenSimplex::new(seed),
        }
    }
}

impl NoiseStrategy for AltitudeStrategy {
    fn generate(&self, x: f64, y: f64, detail_level: u32) -> f64 {
        let base_octaves = 6 + detail_level;
        let mountain_octaves = 4 + detail_level;
        let persistence = 0.5;
        let lacunarity = 2.0;

        // Generate base terrain
        let mut amplitude = 1.0;
        let mut frequency = 1.0;
        let mut base_noise = 0.0;
        let mut weight = 0.0;

        for _ in 0..base_octaves {
            let sample_x = x * frequency / self.scale;
            let sample_y = y * frequency / self.scale;

            base_noise += self.get([sample_x, sample_y]) * amplitude;
            weight += amplitude;

            amplitude *= persistence;
            frequency *= lacunarity;
        }
        base_noise /= weight;

        // Add mountainous features
        let mut amplitude = 0.5;
        let mut frequency = 1.0;
        let mut mountain_noise = 0.0;
        let mut weight = 0.0;

        for _ in 0..mountain_octaves {
            let sample_x = x * frequency / self.mountain_scale;
            let sample_y = y * frequency / self.mountain_scale;

            mountain_noise += self.noise.get([sample_x, sample_y]) * amplitude;
            weight += amplitude;

            amplitude *= persistence;
            frequency *= lacunarity;
        }
        mountain_noise /= weight;

        // Combine base terrain with mountains using exponential function
        let combined = base_noise + (mountain_noise * mountain_noise * 2.0);
        combined.max(-1.0).min(1.0)
    }
}


pub trait NoiseStrategy {
    fn generate(&self, x: f64, y: f64, detail_level: u32) -> f64;
}

pub struct ContinentalnessStrategy {
    scale: f64,
    noise: OpenSimplex,
}

impl ContinentalnessStrategy {
    pub fn new(seed: u32) -> Self {
        Self {
            scale: 100.0,
            noise: OpenSimplex::new(seed),
        }
    }
}

impl NoiseStrategy for ContinentalnessStrategy {
    fn generate(&self, x: f64, y: f64, detail_level: u32) -> f64 {
        let octaves = 4 + detail_level;
        let persistence = 0.5;
        let lacunarity = 2.0;

        let mut amplitude = 1.0;
        let mut frequency = 1.0;
        let mut noise_value = 0.0;
        let mut weight = 0.0;

        for _ in 0..octaves {
            let sample_x = x * frequency / self.scale;
            let sample_y = y * frequency / self.scale;

            noise_value += self.noise.get([sample_x, sample_y]) * amplitude;
            weight += amplitude;

            amplitude *= persistence;
            frequency *= lacunarity;
        }

        noise_value / weight
    }
}

pub struct TemperatureStrategy {
    scale: f64,
    latitude_influence: f64,
    noise: OpenSimplex,
}

impl TemperatureStrategy {
    pub fn new(seed: u32) -> Self {
        Self {
            scale: 150.0,
            latitude_influence: 0.7,
            noise: OpenSimplex::new(seed),
        }
    }
}

impl NoiseStrategy for TemperatureStrategy {
    fn generate(&self, x: f64, y: f64, detail_level: u32) -> f64 {
        let octaves = 3 + detail_level;
        let persistence = 0.6;
        let lacunarity = 2.5;

        let mut amplitude = 1.0;
        let mut frequency = 1.0;
        let mut noise_value = 0.0;
        let mut weight = 0.0;

        for _ in 0..octaves {
            let sample_x = x * frequency / self.scale;
            let sample_y = y * frequency / self.scale;

            noise_value += self.noise.get([sample_x, sample_y]) * amplitude;
            weight += amplitude;

            amplitude *= persistence;
            frequency *= lacunarity;
        }

        let normalized_noise = noise_value / weight;
        let latitude_factor = y * 2.0 - 1.0;

        (normalized_noise * (1.0 - self.latitude_influence) +
            latitude_factor * self.latitude_influence) * 100.0
    }
}
