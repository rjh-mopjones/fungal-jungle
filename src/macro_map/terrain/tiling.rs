use image::Rgb;
use crate::macro_map::terrain::noise_layers::NoiseValues;

#[derive(Debug, Clone, Copy)]
pub struct ThresholdRange {
    pub min: f64,
    pub max: f64,
}
impl ThresholdRange {
    pub fn new(min: f64, max: f64) -> Self {
        Self { min, max }
    }

    pub fn contains(&self, value: f64) -> bool {
        value >= self.min && value <= self.max
    }
}

pub struct TilingConfig {
    pub sea_level: f64,
    pub river_threshold: f64,
}

impl Default for TilingConfig {
    fn default() -> Self {
        Self {
            sea_level: 0.0,
            river_threshold: 0.8,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TileThresholds {
    pub altitude: ThresholdRange,
    pub temperature: ThresholdRange,
    pub continentalness: ThresholdRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tile {
    Sea,
    White,
    Snow,
    Mountain,
    Forest,
    Plains,
    Basin,
    Plateau,
    Desert,
    Beach,
    Black
}

impl Tile {
    pub(crate) fn index(&self) -> u8{
        match *self {
            Tile::Sea => 0,
            Tile::Plains => 1,
            Tile::White => 2,
            Tile::Snow => 3,
            Tile::Forest=> 4,
            Tile::Basin => 5,
            Tile::Desert => 6,
            Tile::Mountain=> 7,
            Tile::Plateau=> 8,
            Tile::Beach=> 9,
            _ => 0
        }
    }
}

impl Tile {
    pub(crate) fn u8colour(&self) -> [u8; 4]{
        match *self {
            Tile::Sea =>[0,191,255,255] ,
            Tile::Plains => [50, 205, 50, 255],
            Tile::White => [255, 255, 255, 255],
            Tile::Snow => [211,211,211, 255],
            Tile::Forest=> [0, 100, 0, 255],
            Tile::Basin => [255,215,0, 255],
            Tile::Desert => [255,165,0, 255],
            Tile::Mountain=> [105,105,105, 255],
            Tile::Plateau=> [139,69,19, 255],
            Tile::Beach=> [222,184,135, 255],
            _ => {[0,0,0,255]}
        }
    }
}

impl Tile {
    pub(crate) fn rbg_colour(&self) -> Rgb<u8>{
        match *self {
            Tile::Sea =>Rgb([0,191,255]),
            Tile::Plains => Rgb([50, 205, 50]),
            Tile::White => Rgb([255, 255, 255]),
            Tile::Snow => Rgb([211,211,211]),
            Tile::Forest=> Rgb([0, 100, 0]),
            Tile::Basin => Rgb([255,215,0]),
            Tile::Desert => Rgb([255,165,0]),
            Tile::Mountain=> Rgb([105,105,105]),
            Tile::Plateau=> Rgb([139,69,19]),
            Tile::Beach=> Rgb([222,184,135]),
            _ => Rgb([0,0,0])
        }
    }
}

impl Tile {
    pub fn thresholds(&self, config: &TilingConfig) -> TileThresholds {
        match self {
            Tile::Sea => TileThresholds {
                altitude: ThresholdRange::new(f64::MIN, config.sea_level),
                temperature: ThresholdRange::new(f64::MIN, f64::MAX),
                continentalness: ThresholdRange::new(0.0, config.river_threshold),
            },
            Tile::Mountain => TileThresholds {
                altitude: ThresholdRange::new(config.sea_level + 0.7, f64::MAX),
                temperature: ThresholdRange::new(f64::MIN, f64::MAX),
                continentalness: ThresholdRange::new(0.0, f64::MAX),
            },
            Tile::Beach => TileThresholds {
                altitude: ThresholdRange::new(config.sea_level, config.sea_level + 0.1),
                temperature: ThresholdRange::new(f64::MIN, f64::MAX),
                continentalness: ThresholdRange::new(0.0, config.river_threshold),
            },
            Tile::Basin => TileThresholds {
                altitude: ThresholdRange::new(f64::MIN, config.sea_level),
                temperature: ThresholdRange::new(70.0, f64::MAX),
                continentalness: ThresholdRange::new(f64::MIN, f64::MAX),
            },
            Tile::Snow => TileThresholds {
                altitude: ThresholdRange::new(config.sea_level, config.sea_level + 0.7),
                temperature: ThresholdRange::new(f64::MIN, -30.0),
                continentalness: ThresholdRange::new(0.0, f64::MAX),
            },
            Tile::Forest => TileThresholds {
                altitude: ThresholdRange::new(config.sea_level + 0.1, config.sea_level + 0.7),
                temperature: ThresholdRange::new(-30.0, 70.0),
                continentalness: ThresholdRange::new(0.6, f64::MAX),
            },
            Tile::Plains => TileThresholds {
                altitude: ThresholdRange::new(config.sea_level + 0.1, config.sea_level + 0.7),
                temperature: ThresholdRange::new(-30.0, 70.0),
                continentalness: ThresholdRange::new(0.0, 0.6),
            },
            Tile::White => TileThresholds {
                altitude: ThresholdRange::new(f64::MIN, config.sea_level),
                temperature: ThresholdRange::new(f64::MIN, -15.0),
                continentalness: ThresholdRange::new(0.0, 0.6),
            },
            Tile::Plateau => TileThresholds {
                altitude: ThresholdRange::new(f64::MIN, config.sea_level + 0.7),
                temperature: ThresholdRange::new(70.0, f64::MAX),
                continentalness: ThresholdRange::new(f64::MIN, f64::MAX),
            },
            Tile::Desert => TileThresholds {
                altitude: ThresholdRange::new(config.sea_level, f64::MAX),
                temperature: ThresholdRange::new(70.0, f64::MAX),
                continentalness: ThresholdRange::new(f64::MIN, f64::MAX),
            },
            Tile::Black => TileThresholds {
                altitude: ThresholdRange::new(f64::MIN, f64::MAX),
                temperature: ThresholdRange::new(f64::MIN, f64::MAX),
                continentalness: ThresholdRange::new(f64::MIN, f64::MAX),
            }
        }
    }

    pub fn matches(&self, config: &TilingConfig, altitude: f64, temp: f64, cont: f64) -> bool {
        let bounds = self.thresholds(config);
        bounds.altitude.contains(altitude) &&
            bounds.temperature.contains(temp) &&
            bounds.continentalness.contains(cont)
    }

    pub fn all() -> Vec<Tile> {
        use Tile::*;
        vec![ Sea, White, Snow, Mountain, Forest, Plains, Basin, Plateau, Desert, Beach, Black ]
    }
}

pub struct TilingStrategy {
    config: TilingConfig,
}

impl TilingStrategy {
    pub(crate) fn get_grayscale_tile(&self, noise: f64) -> Rgb<u8> {
        let value = (noise * 255f64) as u8;
        Rgb([value, value, value])
    }
}

impl TilingStrategy {
    pub fn new(config: TilingConfig) -> Self {
        Self { config }
    }

    pub fn get_tile(&self, noise_values: &NoiseValues) -> Tile {
        Tile::all().into_iter()
            .find(|tile| tile.matches(&self.config, noise_values.altitude, noise_values.temperature, noise_values.continentalness))
            .unwrap_or(Tile::Black)
    }
}