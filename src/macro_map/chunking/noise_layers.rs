use noise::{NoiseFn, OpenSimplex};

#[derive(Default)]
pub struct NoiseValues {
    continentalness: f64,
    temperature: f64,
}

pub struct NoiseStrategies { pub continentalness_strategy: ContinentalnessStrategy,
    pub temperature_strategy: TemperatureStrategy
}
impl NoiseStrategies {
    pub fn generate(&self, x: f64, y: f64, detail_level: u32) -> NoiseValues {
        NoiseValues {
            continentalness: self.continentalness_strategy.generate(x, y, detail_level),
            temperature: self.temperature_strategy.generate(x, y, detail_level),
        }
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
