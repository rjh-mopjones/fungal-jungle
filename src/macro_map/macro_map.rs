use rayon::iter::plumbing::{bridge, Consumer, Producer, ProducerCallback, UnindexedConsumer};
use rayon::prelude::*;
use crate::macro_map::tile::Tile;

pub type Colour = [u8; 4];
#[derive(Copy,Clone)]
pub struct MacroMapTile {
    pub(crate) tile: Tile,
    pub(crate) temperature: f64,
    pub(crate) height: f64,
    pub(crate) coords: (f64, f64),
}

#[derive(Clone)]
pub struct MacroMapLine{
    pub(crate) map: Vec<MacroMapTile>
}

pub struct MacroMap {
    pub(crate) size: (usize, usize),
    pub(crate) border_value: f64,
    pub(crate) map: Vec<MacroMapLine>
}

pub struct ParDataIter<'a> {
    data_slice: &'a [MacroMapLine]
}
struct DataProducer<'a> {
    data_slice : &'a [MacroMapLine],
}

impl<'a> ParallelIterator for ParDataIter<'a> {
    type Item = &'a MacroMapLine;

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item> {
        bridge(self,consumer)
    }

    fn opt_len(&self) -> Option<usize> {
        Some(self.len())
    }}

impl<'a> IndexedParallelIterator for ParDataIter<'a> {
    fn len(&self) -> usize {
        self.data_slice.len()
    }

    fn drive<C: Consumer<Self::Item>>(self, consumer: C) -> C::Result {
        bridge(self,consumer)
    }

    fn with_producer<CB: ProducerCallback<Self::Item>>(
        self,
        callback: CB,
    ) -> CB::Output {
        let producer = DataProducer::from(self);
        callback.callback(producer)
    }
}

impl<'a> Producer for DataProducer<'a> {
    type Item = &'a MacroMapLine;
    type IntoIter = std::slice::Iter<'a, MacroMapLine>;

    fn into_iter(self) -> Self::IntoIter {
        self.data_slice.iter()
    }

    fn split_at(self, index: usize) -> (Self, Self) {
        let (left, right) = self.data_slice.split_at(index);
        (
            DataProducer { data_slice: left },
            DataProducer { data_slice: right },
        )
    }
}

impl<'a> From<ParDataIter<'a>> for DataProducer<'a> {
    fn from(iterator: ParDataIter<'a>) -> Self {
        Self {
            data_slice: iterator.data_slice,
        }
    }
}
impl MacroMap {
    pub fn parallel_iterator(&self) -> ParDataIter {
        ParDataIter {
            data_slice : &self.map,
        }
    }
}

impl<'a> IntoParallelIterator for &'a MacroMap{
    type Iter = ParDataIter<'a>;
    type Item = &'a MacroMapLine;

    fn into_par_iter(self) -> Self::Iter {
        ParDataIter { data_slice: &self.map}
    }
}

pub fn generate_macro_map<G: crate::jungle_noise::generator::Generator<3> + Sync>(sea_level: f64, width: usize,
                                                                           height:usize, zoom: f64,
                                                                           centre: (f64, f64), generator: &G) -> MacroMap {
    let x_origin = centre.0 * width as f64;
    let y_origin = centre.1 * height as f64;

    let x_extent = (width as f64 - (x_origin))/zoom;
    let y_extent = (height as f64 - (y_origin))/zoom;

    let x_step = x_extent / width as f64;
    let y_step = y_extent / height as f64;

    let blank_tile = MacroMapTile {
        tile: Tile::Blank,
        temperature: 0.0,
        height: 0.0,
        coords:(0.0,0.0)
    };

    let blank_line = MacroMapLine{
        map: vec![blank_tile; width]
    };

    let mut macro_map = MacroMap {
        size: (width, height),
        border_value: 0.0,
        map: vec![blank_line.clone(); height]
    };

    let results = macro_map.map
        .par_iter()
        .enumerate()
        .map(|(y, line)| {
            let current_y = y_origin + y_step * y as f64;
            let mut new_line = MacroMapLine {
                map: vec![blank_tile; width]
            };
            for x in 0..width {
                let current_x = x_origin + x_step * x as f64;
                let mut sample: f64 = generator.sample([current_x, current_y, 1.0]);
                let mut m_temperature: f64 = (((current_y / height as f64) * 150.0) - 50.0)
                    + 100.0 * generator.sample([current_x, current_y, 1.1]);
                new_line.map[x] = MacroMapTile {
                    tile: create_tile(sea_level, sample, m_temperature),
                    temperature: m_temperature,
                    height: sample,
                    coords: (x as f64, y as f64)
                }
            }
            return new_line;
        }).collect();

    MacroMap {
        size: (width, height),
        border_value: 0.0,
        map: results
    }
}

fn create_tile(sea_level: f64, sample: f64, temperature: f64) -> Tile {
    return if sample < sea_level {
        if temperature < -15.0 {
            Tile::Ice
        } else if temperature > 50.0 {
            Tile::Desert
        } else {
            Tile::Sea
        }
    } else if sample < sea_level + 0.02 {
        if temperature > 3.0 {
            Tile::Beach
        } else {
            Tile::Snow
        }
    } else if sample < sea_level + 0.1 {
        if temperature < 3.0 {
            Tile::Snow
        } else if temperature > 60.0 {
            Tile::Sahara
        } else {
            Tile::Plains
        }
    } else if sample < sea_level + 0.2 {
        if temperature < 3.0 {
            Tile::Snow
        } else if temperature > 60.0 {
            Tile::Sahara
        } else {
            Tile::Forest
        }
    } else if sample < sea_level + 0.3 {
        if temperature > 70.0 {
            Tile::Plateau
        } else {
            Tile::Mountain
        }
    } else {
        if temperature < 70.0 {
            Tile::Snow
        } else {
            Tile::Plateau
        }
    }
}

pub fn write_macro_map_to_file(macro_map: MacroMap, filename: &str) {
    let (width, height) = macro_map.size;
    let mut result = Vec::with_capacity(height * width);

    for y in 0..height {
        for x in 0..width {
            for z in macro_map.map[y].map[x].tile.u8colour(){
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
