use bevy::prelude::Color;
use bevy::prelude::Color::Rgba;

#[derive(Copy,Clone)]
pub enum Tile {
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
    pub(crate) fn normal_colour(&self) -> Color{
        match *self {
            Tile::Sea => Rgba { red: 0.0, green: 0.749, blue: 1.0, alpha: 1.0},
            Tile::Plains => Rgba { red: 0.196, green: 0.804, blue: 0.196, alpha: 1.0},
            Tile::Ice => Rgba { red: 1.0, green: 1.0, blue: 1.0, alpha: 1.0},
            Tile::Snow => Rgba { red: 0.828, green: 0.828, blue: 0.828, alpha: 1.0},
            Tile::Forest => Rgba { red: 0.0, green: 0.392, blue: 0.0, alpha: 1.0},
            Tile::Desert => Rgba { red: 1.0, green: 0.843, blue: 0.0, alpha: 1.0},
            Tile::Sahara => Rgba { red: 1.0, green: 0.647, blue: 0.0, alpha: 1.0},
            Tile::Mountain=> Rgba { red: 0.412, green: 0.412, blue: 0.412, alpha: 1.0},
            Tile::Plateau=> Rgba { red: 0.412, green: 0.271, blue: 0.075, alpha: 1.0},
            Tile::Beach=> Rgba { red: 0.871, green: 0.723, blue: 0.529, alpha: 1.0},
            _ => Rgba { red: 0.0, green: 0.0, blue: 0.0, alpha: 0.0},
        }
    }
}
impl Tile{
    fn index(&self) -> u8{
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
