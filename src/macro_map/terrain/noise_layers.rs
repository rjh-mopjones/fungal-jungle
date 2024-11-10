use image::{DynamicImage, GenericImage, Pixel};
use noise::{NoiseFn, OpenSimplex};
use crate::macro_map::terrain::tiling::TilingStrategy;

#[derive(Default, Clone)]
pub struct NoiseValues {
    pub(crate) continentalness: f64,
    pub(crate) temperature: f64,
    pub(crate) altitude: f64
}

#[derive(Default)]
pub struct NoiseLayers {
    pub(crate) aggregate: DynamicImage,
    pub(crate) continentalness: DynamicImage,
    pub(crate) temperature: DynamicImage,
    pub(crate) altitude: DynamicImage
}

impl NoiseLayers {
    pub(crate) fn new(size: usize) -> Self {
        Self {
            aggregate: DynamicImage::new_rgb8(size as u32, size as u32),
            continentalness: DynamicImage::new_rgb8(size as u32, size as u32),
            temperature: DynamicImage::new_rgb8(size as u32, size as u32),
            altitude: DynamicImage::new_rgb8(size as u32, size as u32),
        }
    }
}

impl NoiseLayers {
    pub(crate) fn add_at_index(&mut self, x: usize, y: usize, noise_values: &NoiseValues, tiling_strategy: &TilingStrategy) {
        self.aggregate.put_pixel(x as u32, y as u32, tiling_strategy.get_tile(noise_values).rbg_colour().to_rgba());
        self.continentalness.put_pixel(x as u32, y as u32, tiling_strategy.get_grayscale_tile(noise_values.continentalness).to_rgba());
        self.temperature.put_pixel(x as u32, y as u32, tiling_strategy.get_grayscale_tile(noise_values.temperature).to_rgba());
        self.altitude.put_pixel(x as u32, y as u32, tiling_strategy.get_grayscale_tile(noise_values.altitude).to_rgba());
    }
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

            base_noise += self.noise.get([sample_x, sample_y]) * amplitude;
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
