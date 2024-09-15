use image::Rgb;
use crate::macro_map::macromap::TileLayer;

#[derive(Default, Copy, Clone, Debug)]
pub enum TileType {
    #[default]
    Sea,
    White,
    Snow,
    Jungle,
    Mountain,
    Forest,
    Plains,
    Desert,
    Plateau,
    Sahara,
    Beach,
    Black
}

impl TileType {
    pub(crate) fn index(&self) -> u8{
        match *self {
            TileType::Sea => 0,
            TileType::Plains => 1,
            TileType::White => 2,
            TileType::Snow => 3,
            TileType::Forest=> 4,
            TileType::Desert=> 5,
            TileType::Sahara=> 6,
            TileType::Mountain=> 7,
            TileType::Plateau=> 8,
            TileType::Beach=> 9,
            _ => 0
        }
    }
}

impl TileType {
    pub(crate) fn u8colour(&self) -> [u8; 4]{
        match *self {
            TileType::Sea =>[0,191,255,255] ,
            TileType::Plains => [50, 205, 50, 255],
            TileType::White => [255, 255, 255, 255],
            TileType::Snow => [211,211,211, 255],
            TileType::Forest=> [0, 100, 0, 255],
            TileType::Desert=> [255,215,0, 255],
            TileType::Sahara=> [255,165,0, 255],
            TileType::Mountain=> [105,105,105, 255],
            TileType::Plateau=> [139,69,19, 255],
            TileType::Beach=> [222,184,135, 255],
            _ => {[0,0,0,255]}
        }
    }
}

impl TileType {
    pub(crate) fn rbg_colour(&self) -> Rgb<u8>{
        match *self {
            TileType::Sea =>Rgb([0,191,255]),
            TileType::Plains => Rgb([50, 205, 50]),
            TileType::White => Rgb([255, 255, 255]),
            TileType::Snow => Rgb([211,211,211]),
            TileType::Forest=> Rgb([0, 100, 0]),
            TileType::Desert=> Rgb([255,215,0]),
            TileType::Sahara=> Rgb([255,165,0]),
            TileType::Mountain=> Rgb([105,105,105]),
            TileType::Plateau=> Rgb([139,69,19]),
            TileType::Beach=> Rgb([222,184,135]),
            _ => Rgb([0,0,0])
        }
    }
}

pub fn get_tile(sea_level: f64, tile_layer: TileLayer) -> TileType {
    return if tile_layer.continentalness < sea_level {
        if tile_layer.temperature < -15.0 {
            TileType::White
        } else if tile_layer.temperature > 50.0 {
            TileType::Desert
        } else {
            TileType::Sea
        }
    } else if tile_layer.continentalness < sea_level + 0.02 {
        if tile_layer.temperature > 3.0 {
            TileType::Beach
        } else {
            TileType::Snow
        }
    } else if tile_layer.continentalness < sea_level + 0.1 {
        if tile_layer.temperature < 3.0 {
            TileType::Snow
        } else if tile_layer.temperature > 60.0 {
            TileType::Sahara
        } else {
            TileType::Plains
        }
    } else if tile_layer.continentalness < sea_level + 0.2 {
        if tile_layer.temperature < 3.0 {
            TileType::Snow
        } else if tile_layer.temperature > 60.0 {
            TileType::Sahara
        } else {
            TileType::Forest
        }
    } else if tile_layer.continentalness < sea_level + 0.3 {
        if tile_layer.temperature > 70.0 {
            TileType::Plateau
        } else {
            TileType::Mountain
        }
    } else {
        if tile_layer.temperature < 70.0 {
            TileType::Snow
        } else {
            TileType::Plateau
        }
    }
}
