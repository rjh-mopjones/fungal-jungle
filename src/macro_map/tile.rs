use bevy::prelude::Color;
use image::Rgb;

#[derive(Default, Copy, Clone, Debug)]
pub enum Tile {
    #[default]
    Sea,
    Ice,
    Snow,
    Jungle,
    Mountain,
    Forest,
    Plains,
    Desert,
    Plateau,
    Sahara,
    Beach,
    Blank,
}

impl Tile{
    pub(crate) fn index(&self) -> u8{
        match *self {
            Tile::Sea => 0,
            Tile::Plains => 1,
            Tile::Ice => 2,
            Tile::Snow => 3,
            Tile::Forest=> 4,
            Tile::Desert=> 5,
            Tile::Sahara=> 6,
            Tile::Mountain=> 7,
            Tile::Plateau=> 8,
            Tile::Beach=> 9,
            _ => 0
        }
    }
}

impl Tile{
    pub(crate) fn u8colour(&self) -> [u8; 4]{
        match *self {
            Tile::Sea =>[0,191,255,255] ,
            Tile::Plains => [50, 205, 50, 255],
            Tile::Ice => [255, 255, 255, 255],
            Tile::Snow => [211,211,211, 255],
            Tile::Forest=> [0, 100, 0, 255],
            Tile::Desert=> [255,215,0, 255],
            Tile::Sahara=> [255,165,0, 255],
            Tile::Mountain=> [105,105,105, 255],
            Tile::Plateau=> [139,69,19, 255],
            Tile::Beach=> [222,184,135, 255],
            _ => {[0,0,0,255]}
        }
    }
}

impl Tile{
    pub(crate) fn rbg_colour(&self) -> Rgb<u8>{
        match *self {
            Tile::Sea =>Rgb([0,191,255]),
            Tile::Plains => Rgb([50, 205, 50]),
            Tile::Ice => Rgb([255, 255, 255]),
            Tile::Snow => Rgb([211,211,211]),
            Tile::Forest=> Rgb([0, 100, 0]),
            Tile::Desert=> Rgb([255,215,0]),
            Tile::Sahara=> Rgb([255,165,0]),
            Tile::Mountain=> Rgb([105,105,105]),
            Tile::Plateau=> Rgb([139,69,19]),
            Tile::Beach=> Rgb([222,184,135]),
            _ => Rgb([0,0,0])
        }
    }
}
