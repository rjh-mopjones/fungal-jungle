use crate::macro_map::macro_map::Tile::{Plains, Sea};

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
    Blank,
}
#[derive(Copy,Clone)]
pub struct MacroMapTile {
    pub(crate) tile: Tile,
    pub(crate) temperature: f64,
    pub(crate) height: f64,
}

pub struct MacroMap {
    pub(crate) size: (usize, usize),
    pub(crate) border_value: f64,
    pub(crate) map: Vec<Vec<MacroMapTile>>
}

pub fn generate_macro_map<G: crate::jungle_noise::generator::Generator<2>>(width: usize, height:usize, generator: &G) -> MacroMap {

    let blank_tile = MacroMapTile {
        tile: Tile::Blank,
        temperature: 0.0,
        height: 0.0
    };

    let mut macro_map = MacroMap {
        size: (width, height),
        border_value: 0.0,
        map: vec![vec![blank_tile; width]; height]
    };
    for y in 0..height {
        for x in 0..width {
            let mut output: f64 = generator.sample([x as f64, y as f64]);

            if output < 0.0 {
                macro_map.map[x][y].tile = Sea;
                macro_map.map[x][y].temperature = ((y/height) as f64 * 150.0) - 50.0;
            } else {
                macro_map.map[x][y].tile = Plains;
                macro_map.map[x][y].temperature = ((y/height) as f64 * 150.0) - 50.0;
            }
        }
    }

    return macro_map;
}