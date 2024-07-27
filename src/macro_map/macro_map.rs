use std::time::Instant;
use bevy::math::{Vec2, vec2};
use bevy::utils::default;
use rayon::iter::plumbing::{bridge, Consumer, Producer, ProducerCallback, UnindexedConsumer};
use rayon::prelude::*;
use crate::macro_map::tile::Tile;
use image::{DynamicImage, GenericImage, ImageBuffer, RgbImage}; use bevy::prelude::*;
use crate::jungle_noise::generator::Generator;


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
    pub(crate) high_res_map: Vec<MacroMapTile>,
    pub(crate) low_res_dynamic_image: DynamicImage
}

#[derive(Default, Clone, Debug)]
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

pub fn generate_macro_map(width: usize, height:usize, zoom: usize, seed: u64, grayscale: bool) -> MacroMap {

    const CONTINENT_FREQUENCY: f64 = 1.00;
    const CONTINENT_LACUNARITY: f64 = 2.00;
    const SEA_LEVEL: f64 = -0.025;

    // Do fbm perlin for base continent def
    let generator = crate::jungle_noise::source::Source::<3>::improved_perlin(seed).scale([0.01; 3])
        .fbm(16, CONTINENT_FREQUENCY, CONTINENT_LACUNARITY, 0.59);


    let now = Instant::now();

    let meso_x = width / MESO_LOW_RES_PIXELS;
    let meso_y = height / MESO_LOW_RES_PIXELS;
    let total_meso_tiles = MESO_LOW_RES_PIXELS * MESO_LOW_RES_PIXELS;

    let blank_tile = MacroMapTile {
        tile: Tile::Black,
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
            println!("On meso map {}/64, {}/32", i, j);
            let mut new_meso_map = MesoMap {
                index : vec2(i as f32, j as f32),
                location : vec2(((i * 16) + 8 )as f32, ((j * 16) + 8) as f32),
                scale : vec2(1.0/ MESO_LOW_RES_PIXELS as f32, 1.0/ MESO_LOW_RES_PIXELS as f32),
                is_high_res_loaded: false,
                low_res_map: vec![blank_tile; total_meso_tiles * zoom],
                high_res_map: vec![],
                low_res_dynamic_image: DynamicImage::default()
            };
            let mut img: RgbImage = ImageBuffer::new(MESO_LOW_RES_PIXELS as u32 * zoom as u32,  MESO_LOW_RES_PIXELS as u32 * zoom as u32);

            for tile_idx in 0..total_meso_tiles {
                let (local_x, local_y) = div_rem_usize(tile_idx, MESO_LOW_RES_PIXELS);
                let global_x = (new_meso_map.location.x as usize + local_x) as f64;
                let global_y = (new_meso_map.location.y as usize + local_y) as f64;
                let zoomed_tile_idx = tile_idx * zoom;
                let zoomed_x = local_x * zoom;
                let zoomed_y = local_y * zoom;

                for x_step in 0..zoom {
                    for y_step in 0..zoom{
                        let x_extent :f64 = (1.0 / zoom as f64) * x_step as f64;
                        let y_extent :f64 = (1.0 / zoom as f64) * y_step as f64;
                        let mut sample: f64 = generator.sample([global_x + x_extent, global_y+y_extent, 1.0]);
                        let mut m_temperature: f64 = ((((global_y+y_extent)/ height as f64) * 150.0) - 50.0)
                            + 100.0 * generator.sample([global_x+x_extent, global_y+y_extent, 1.1]);
                        let mut m_tile = create_tile(SEA_LEVEL, sample, m_temperature);
                        let x_pixel = MESO_LOW_RES_PIXELS as u32 * zoom as u32 - zoomed_x as u32 - x_step as u32 - 1u32;
                        let y_pixel = zoomed_y as u32 + y_step as u32;

                        img.put_pixel(x_pixel,
                                      y_pixel, m_tile.rbg_colour());
                        new_meso_map.low_res_map[zoomed_tile_idx + y_step] = MacroMapTile {
                            tile: m_tile,
                            temperature: m_temperature,
                            height: sample,
                            coords: (global_x, global_y)
                        };

                    }

                }

            }
            new_meso_map.low_res_dynamic_image = DynamicImage::from(img).to_rgba8().into();
            return new_meso_map;
        }).collect();

    let elapsed = now.elapsed();
    println!("Time to generate MacroMap: {:.2?}", elapsed);
    MacroMap {
        size: (width, height),
        meso_maps: results,
        meso_pixels: MESO_LOW_RES_PIXELS,
    }
}

fn create_tile(sea_level: f64, sample: f64, temperature: f64) -> Tile {
    return if sample < sea_level {
        if temperature < -15.0 {
            Tile::White
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

pub fn write_meso_map_to_file(macro_map: MacroMap, index :usize, filename: &str) {
    let image : DynamicImage = macro_map.meso_maps[index].low_res_dynamic_image.clone();
    image.save(filename).unwrap();
}

pub fn write_macro_map_to_file(macro_map: MacroMap, filename: &str) {
    let cols = 64;
    let rows = 32;
    let meso_map_res = macro_map.meso_maps[0].low_res_map.len()/2;
    let mut combined_image = ImageBuffer::new((meso_map_res as u32 * cols as u32), (meso_map_res as u32 * rows as u32));

    // Iterate over the images and place them in the combined image
    for (idx, meso_map) in macro_map.meso_maps.iter().enumerate(){
        let (i, j) = div_rem_usize(idx, rows);

        let x_offset = i as u32 * meso_map_res as u32;
        let y_offset = j as u32 * meso_map_res as u32;

        // Copy each image into the combined image at the correct position
        combined_image.copy_from(&meso_map.low_res_dynamic_image, x_offset, y_offset).unwrap();
    }
    combined_image.save(filename).unwrap();
}

pub fn div_rem<T: std::ops::Div<Output=T> + std::ops::Rem<Output=T> + Copy>(x: T, y: T) -> (T, T) {
    let quot = x / y;
    let rem = x % y;
    (quot, rem)
}

pub fn div_rem_usize(x: usize, y: usize) -> (usize, usize) {
    div_rem(x, y)
}
