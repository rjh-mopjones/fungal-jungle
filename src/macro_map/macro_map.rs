use crate::macro_map::macro_map::Tile::{Plains, Sea};

pub type Color = [u8; 4];
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
    pub(crate) colour: Color,

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
        height: 0.0,
        colour: [  0,   0,   0, 255]
    };

    let mut macro_map = MacroMap {
        size: (width, height),
        border_value: 0.0,
        map: vec![vec![blank_tile; height]; width]
    };
    for y in 0..height {
        for x in 0..width {
            let mut output: f64 = generator.sample([x as f64, y as f64]);

            if output < 0.0 {
                macro_map.map[x][y].tile = Sea;
                macro_map.map[x][y].temperature = ((y/height) as f64 * 150.0) - 50.0;
                macro_map.map[x][y].colour = [  0,   191,   255, 255];
                macro_map.map[x][y].height = output;
            } else {
                macro_map.map[x][y].tile = Plains;
                macro_map.map[x][y].temperature = ((y/height) as f64 * 150.0) - 50.0;
                macro_map.map[x][y].colour = [34, 139, 34, 255];
                macro_map.map[x][y].height = output;
            }
        }
    }

    return macro_map;
}

pub fn write_macro_map_to_file(macro_map: MacroMap, filename: &str) {
    let (width, height) = macro_map.size;
    let mut result = Vec::with_capacity(width * height);

    for i in &macro_map.map {
        for j in i.iter() {
            for k in j.colour {
                result.push(k);
            }
        }
    }
    let _ = image::save_buffer(
        filename,
        &result,
        width as u32,
        height as u32,
        image::ColorType::Rgba8,
    );
}
