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
    Sahara,
    Blank,
}
impl Tile{
    fn colour(&self) -> Color{
        match *self {
            Tile::Sea =>[0,191,255,255] ,
            Tile::Plains => [50, 205, 50, 255],
            Tile::Ice => [255, 255, 255, 255],
            Tile::Snow => [250,235,215, 255],
            Tile::Forest=> [0, 100, 0, 255],
            Tile::Desert=> [255,215,0, 255],
            Tile::Sahara=> [255,165,0, 255],
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
            let mut temp_sample: f64 = generator.sample([x as f64, y as f64, 1.1]);

            let mut temperature: f64 = (( y as f64 / height as f64) * 100.0) - 50.0 * temp_sample;
            if y != 0 {
                println!("temp: {}", temperature);
            }

            if sample < 0.0 {
                if temperature < -5.0{
                    macro_map.map[x][y].tile = Tile::Ice;
                } else if temperature > 60.0 {
                    macro_map.map[x][y].tile = Tile::Desert;
                } else {
                    macro_map.map[x][y].tile = Tile::Sea;
                }
            } else {
                if temperature < -5.0 {
                    macro_map.map[x][y].tile = Tile::Snow;
                } else if temperature < 10.0 {
                    macro_map.map[x][y].tile = Tile::Forest;
                } else if temperature > 60.0 {
                    macro_map.map[x][y].tile = Tile::Sahara;
                } else {
                    macro_map.map[x][y].tile = Tile::Plains;
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
