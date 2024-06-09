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
    Plateau,
    Sahara,
    Beach,
    Blank,
}
impl Tile{
    fn colour(&self) -> Color{
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

pub fn generate_macro_map<G: crate::jungle_noise::generator::Generator<3>>(width: usize, height:usize, generator: &G) -> MacroMap {

    let blank_tile = MacroMapTile {
        tile: Tile::Blank,
        temperature: 0.0,
        height: 0.0,
    };

    let mut macro_map = MacroMap {
        size: (width, height),
        border_value: 0.0,
        map: vec![vec![blank_tile; height]; width]
    };
    for y in 0..height{
        for x in 0..width{
            let mut sample: f64 = generator.sample([x as f64, y as f64, 1.0]);
            let mut temperature: f64 = ((( y as f64 / height as f64) * 150.0) - 50.0)
                + 100.0 * generator.sample([x as f64, y as f64, 1.1]);
            if y != 0 {
                println!("sample: {}", sample);
            }

            if sample < 0.0 {
                if temperature < -15.0 {
                    macro_map.map[x][y].tile = Tile::Ice;
                } else if temperature > 50.0 {
                    macro_map.map[x][y].tile = Tile::Desert;
                } else {
                    macro_map.map[x][y].tile = Tile::Sea;
                }
            } else if sample < 0.02 {
                if temperature > 3.0 {
                    macro_map.map[x][y].tile = Tile::Beach;
                } else {
                    macro_map.map[x][y].tile = Tile::Snow;
                }
            } else if sample < 0.1 {
                if temperature < 3.0 {
                    macro_map.map[x][y].tile = Tile::Snow;
                } else if temperature > 60.0 {
                    macro_map.map[x][y].tile = Tile::Sahara;
                } else {
                    macro_map.map[x][y].tile = Tile::Plains;
                }
            } else if sample < 0.2 {
                if temperature < 3.0 {
                    macro_map.map[x][y].tile = Tile::Snow;
                } else if temperature > 60.0 {
                    macro_map.map[x][y].tile = Tile::Sahara;
                } else {
                    macro_map.map[x][y].tile = Tile::Forest;
                }
            } else if sample < 0.3 {
                if temperature > 70.0 {
                    macro_map.map[x][y].tile = Tile::Plateau;
                } else {
                    macro_map.map[x][y].tile = Tile::Mountain;
                }
            } else {
                if temperature < 70.0 {
                    macro_map.map[x][y].tile = Tile::Snow;
                } else {
                    macro_map.map[x][y].tile = Tile::Plateau;
                }
            }
            macro_map.map[x][y].temperature = temperature;
            macro_map.map[x][y].height = sample;
        }
    }

    return macro_map;
}

pub fn write_macro_map_to_file(macro_map: MacroMap, filename: &str) {
    let (width, height) = macro_map.size;
    let mut result = Vec::with_capacity(height * width);

    for y in 0..height {
        for x in 0..width {
            for z in macro_map.map[x][y].tile.colour(){
                result.push(z);
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
