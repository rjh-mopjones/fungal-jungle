use bevy::math::{Vec2, vec2};
use bevy::utils::default;
use rayon::iter::plumbing::{bridge, Consumer, Producer, ProducerCallback, UnindexedConsumer};
use rayon::prelude::*;
use crate::macro_map::tile::Tile;


const MESO_LOW_RES_PIXELS: usize = 16;
const MESO_HIGH_RES_PIXELS: usize = 512;

pub type Colour = [u8; 4];
#[derive(Default, Copy, Clone, Debug)]
pub struct MacroMapTile {
    pub(crate) tile: Tile,
    pub(crate) temperature: f64,
    pub(crate) height: f64,
    pub(crate) coords: (f64, f64),
}

#[derive(Default, Clone, Debug)]
pub struct MesoMap {
    pub(crate) index:  Vec2,
    pub(crate) location:  Vec2,
    pub(crate) scale: Vec2,
    pub(crate) is_high_res_loaded: bool,
    pub(crate) low_res_map: Vec<MacroMapTile>,
    pub(crate) high_res_map: Vec<MacroMapTile>
}

pub struct MacroMap {
    pub(crate) size: (usize, usize),
    pub(crate) meso_pixels: usize,
    pub(crate) meso_maps: Vec<MesoMap>
}

pub struct ParDataIter<'a> {
    data_slice: &'a [MesoMap]
}
struct DataProducer<'a> {
    data_slice : &'a [MesoMap],
}

impl<'a> ParallelIterator for ParDataIter<'a> {
    type Item = &'a MesoMap;

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
    type Item = &'a MesoMap;
    type IntoIter = std::slice::Iter<'a, MesoMap>;

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
            data_slice : &self.meso_maps,
        }
    }
}

impl<'a> IntoParallelIterator for &'a MacroMap{
    type Iter = ParDataIter<'a>;
    type Item = &'a MesoMap;

    fn into_par_iter(self) -> Self::Iter {
        ParDataIter { data_slice: &self.meso_maps}
    }
}

pub fn generate_macro_map<G: crate::jungle_noise::generator::Generator<3> + Sync>(sea_level: f64, width: usize,
                                                                           height:usize, zoom: f64,
                                                                           centre: (f64, f64), generator: &G) -> MacroMap {

    let meso_x = width / MESO_LOW_RES_PIXELS;
    let meso_y = height / MESO_LOW_RES_PIXELS;
    let total_meso_tiles = MESO_LOW_RES_PIXELS * MESO_LOW_RES_PIXELS;

    let blank_tile = MacroMapTile {
        tile: Tile::Blank,
        temperature: 0.0,
        height: 0.0,
        coords:(0.0,0.0)
    };

    let meso_map = MesoMap {..default()};

    let mut macro_map = MacroMap {
        size: (width, height),
        meso_pixels : MESO_LOW_RES_PIXELS,
        meso_maps: vec![meso_map.clone(); meso_x * meso_y]
    };

    let results = macro_map.meso_maps
        .par_iter()
        .enumerate()
        .map(|(index, meso_map)| {
            let (i, j) = div_rem_usize(index, meso_y);
            let mut new_meso_map = MesoMap {
                index : vec2(i as f32, j as f32),
                location : vec2((i * MESO_LOW_RES_PIXELS) as f32, (j * MESO_LOW_RES_PIXELS) as f32),
                scale : vec2(1.0/ MESO_LOW_RES_PIXELS as f32, 1.0/ MESO_LOW_RES_PIXELS as f32),
                is_high_res_loaded: false,
                low_res_map: vec![blank_tile; total_meso_tiles],
                high_res_map: vec![]
            };
            for tile_idx in 0..total_meso_tiles {
                let (local_x, local_y) = div_rem_usize(tile_idx, MESO_LOW_RES_PIXELS);
                let global_x = (new_meso_map.location.x as usize + local_x) as f64;
                let global_y = (new_meso_map.location.y as usize + local_y) as f64;

                let mut sample: f64 = generator.sample([global_x, global_y, 1.0]);
                let mut m_temperature: f64 = (((global_y/ height as f64) * 150.0) - 50.0)
                    + 100.0 * generator.sample([global_x, global_y, 1.1]);
                new_meso_map.low_res_map[tile_idx] = MacroMapTile {
                    tile: create_tile(sea_level, sample, m_temperature),
                    temperature: m_temperature,
                    height: sample,
                    coords: (global_x, global_y)
                }
            }
            return new_meso_map;
        }).collect();

    MacroMap {
        size: (width, height),
        meso_maps: results,
        meso_pixels: MESO_LOW_RES_PIXELS,
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

    for meso_map in macro_map.meso_maps {
        for low_res_tile in meso_map.low_res_map {
            for channel in low_res_tile.tile.u8colour(){
                result.push(channel);
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

pub fn div_rem<T: std::ops::Div<Output=T> + std::ops::Rem<Output=T> + Copy>(x: T, y: T) -> (T, T) {
    let quot = x / y;
    let rem = x % y;
    (quot, rem)
}

pub fn div_rem_usize(x: usize, y: usize) -> (usize, usize) {
    crate::engine::macro_tilemap::map::div_rem(x, y)
}
