use std::time::Instant;
use bevy::app::{App, Startup};
use bevy::asset::{Assets, Handle};
use bevy::core::Name;
use bevy::hierarchy::BuildChildren;
use bevy::math::{vec2, Vec2};
use bevy::prelude::{Commands, default, Entity, Image, ResMut};
use bevy::render::render_asset::RenderAssetUsages;
use bevy_ecs_tilemap::map::{TilemapId, TilemapSize, TilemapTexture, TilemapTileSize, TilemapType};
use bevy_ecs_tilemap::prelude::{get_tilemap_center_transform, TileBundle, TilePos, TileStorage, TileTextureIndex};
use bevy_ecs_tilemap::TilemapBundle;
use image::{DynamicImage, ImageBuffer, Rgb, RgbImage};
use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use rayon::iter::plumbing::{bridge, Consumer, Producer, ProducerCallback, UnindexedConsumer};
use crate::macro_map::jungle_noise;
use crate::macro_map::jungle_noise::generator::Generator;
use crate::macro_map::tiling_strategy::TileType;

const CONTINENT_FREQUENCY: f64 = 1.00;
const CONTINENT_LACUNARITY: f64 = 2.00;
const SEA_LEVEL: f64 = -0.025;

const MAP_HEIGHT: usize = 512;
const MAP_WIDTH: usize = 1024;

const MESO_LOW_RES_PIXELS: usize = 16;
const SEED: u64 = 42;
// 32 for meso map size
// 256 for micro map size but purps out at 64
const DETAIL_FACTOR: usize = 32;

#[derive(Default, Clone, Debug, Copy)]
pub struct TileLayer {
    pub(crate) continentalness: f64,
    pub(crate) erosion: f64,
    pub(crate) peaks_and_valleys: f64,
    pub(crate) temperature: f64,
    pub(crate) humidity: f64,
    pub(crate) altitude: f64,
    pub(crate) resources: f64
}

#[derive(Default, Clone, Debug)]
pub struct MesoLayerImages {
    pub(crate) continentalness: DynamicImage,
    pub(crate) erosion: DynamicImage,
    pub(crate) peaks_and_valleys: DynamicImage,
    pub(crate) temperature: DynamicImage,
    pub(crate) humidity: DynamicImage,
    pub(crate) altitude: DynamicImage,
    pub(crate) resources: DynamicImage,
}

#[derive(Default, Clone, Debug)]
pub struct Tile {
    // Continentalness, Erosion, Peaks and Valleys, Temperature, Humidity
    // also we will want to add two more: Altitude (plates) and Resources
    pub(crate) tile_type: TileType,
    pub(crate) layer: TileLayer,
    pub(crate) coords: (f64, f64),
}

#[derive(Default, Clone, Debug)]
pub struct MesoMap {
    pub(crate) index: Vec2,
    pub(crate) location: Vec2,
    pub(crate) scale: Vec2,
    pub(crate) tiles: Vec<Tile>,
    pub(crate) image: DynamicImage,
    pub(crate) layer_images: MesoLayerImages
}

#[derive(Clone, Debug)]
pub struct Generators {
    pub(crate) continentalness:  jungle_noise::adapters::Fbm<3, jungle_noise::adapters::Scale<3, jungle_noise::sources::ImprovedPerlin<3>>>,
    // pub(crate) erosion: dyn jungle_noise::generator::Generator3D,
    // pub(crate) peaks_and_valleys: dyn jungle_noise::generator::Generator3D,
    pub(crate) temperature:  jungle_noise::adapters::Fbm<3, jungle_noise::adapters::Scale<3, jungle_noise::sources::ImprovedPerlin<3>>>
    // pub(crate) humidity: dyn jungle_noise::generator::Generator3D,
    // pub(crate) altitude: dyn jungle_noise::generator::Generator3D,
    // pub(crate) resources: dyn jungle_noise::generator::Generator3D,
}

#[derive(Clone, Debug)]
pub struct MacroMap {
    pub(crate) size: (usize, usize),
    pub(crate) meso_pixels: usize,
    pub(crate) meso_maps: Vec<MesoMap>,
    pub(crate) generators: Generators
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

fn create_macro_map(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>
) {

    let continentalness_generator = jungle_noise::source::Source::<3>::improved_perlin(SEED).scale([0.01; 3])
        .fbm(16, CONTINENT_FREQUENCY, CONTINENT_LACUNARITY, 0.59);

    let temperature_generator = jungle_noise::source::Source::<3>::improved_perlin(SEED-10).scale([0.01; 3])
        .fbm(8, CONTINENT_FREQUENCY, CONTINENT_LACUNARITY, 0.59);

    let now = Instant::now();

    let meso_x = MAP_WIDTH / MESO_LOW_RES_PIXELS;
    let meso_y = MAP_HEIGHT / MESO_LOW_RES_PIXELS;
    let total_meso_tiles = MESO_LOW_RES_PIXELS * MESO_LOW_RES_PIXELS;

    let blank_tile = Tile {
        tile_type: TileType::Black,
        ..default()
    };

    let meso_map = MesoMap{..default()};
    let meso_maps = vec![meso_map.clone();meso_x * meso_y];

    let results = meso_maps
        .par_iter()
        .enumerate()
        .map(|(index, meso_map)| {
            let (i, j) = div_rem_usize(index, meso_y);
            println!("On meso map {}/64, {}/32", i, j);
            let mut new_meso_map = MesoMap {
                index : vec2(i as f32, j as f32),
                location : vec2(((i * 16) + 8 )as f32, ((j * 16) + 8) as f32),
                scale : vec2(1.0/ MESO_LOW_RES_PIXELS as f32, 1.0/ MESO_LOW_RES_PIXELS as f32),
                tiles: vec![blank_tile.clone(); total_meso_tiles * DETAIL_FACTOR],
                image: DynamicImage::default(),
                layer_images: MesoLayerImages::default()
            };

            let mut meso_image: RgbImage = ImageBuffer::new(MESO_LOW_RES_PIXELS as u32 * DETAIL_FACTOR as u32,
                                                            MESO_LOW_RES_PIXELS as u32 * DETAIL_FACTOR as u32);

            let mut continentalness_img = meso_image.clone();
            let mut temperature_img = meso_image.clone();


            for tile_idx in 0..total_meso_tiles {
                let (local_x, local_y) = div_rem_usize(tile_idx, MESO_LOW_RES_PIXELS);
                let global_x = (new_meso_map.location.x as usize + local_x) as f64;
                let global_y = (new_meso_map.location.y as usize + local_y) as f64;
                let zoomed_tile_idx = tile_idx * DETAIL_FACTOR;
                let zoomed_x = local_x * DETAIL_FACTOR;
                let zoomed_y = local_y * DETAIL_FACTOR;

                for x_step in 0..DETAIL_FACTOR {
                    for y_step in 0..DETAIL_FACTOR {
                        let x_extent :f64 = (1.0 / DETAIL_FACTOR as f64) * x_step as f64;
                        let y_extent :f64 = (1.0 / DETAIL_FACTOR as f64) * y_step as f64;
                        let x_pixel = MESO_LOW_RES_PIXELS as u32 * DETAIL_FACTOR as u32 - zoomed_x as u32 - x_step as u32 - 1u32;
                        let y_pixel = zoomed_y as u32 + y_step as u32;

                        let tile_layer = TileLayer {
                            continentalness: continentalness_generator.sample([global_x + x_extent, global_y+y_extent, 1.0]),
                            erosion: 0.0,
                            peaks_and_valleys: 0.0,
                            temperature: ((((global_y+y_extent)/ MAP_HEIGHT as f64) * 150.0) - 50.0)
                                            + 100.0 * temperature_generator.sample([global_x+x_extent, global_y+y_extent, 1.1]),
                            humidity: 0.0,
                            altitude: 0.0,
                            resources: 0.0,
                        };

                        let mut tile_type = crate::macro_map::tiling_strategy::get_tile(SEA_LEVEL, tile_layer);

                        new_meso_map.tiles[zoomed_tile_idx + y_step] = Tile {
                            tile_type: Default::default(),
                            coords: (global_x, global_y),
                            layer: tile_layer,
                        };

                        continentalness_img.put_pixel(x_pixel, y_pixel, Rgb([(&tile_layer.continentalness * 255f64) as u8,
                                                                            (&tile_layer.continentalness * 255f64) as u8,
                                                                            (&tile_layer.continentalness * 255f64) as u8]));
                        temperature_img.put_pixel(x_pixel, y_pixel, Rgb([(&tile_layer.continentalness * 255f64) as u8,
                                                                        (&tile_layer.continentalness * 255f64) as u8,
                                                                        (&tile_layer.continentalness * 255f64) as u8]));

                        meso_image.put_pixel(x_pixel, y_pixel, tile_type.rbg_colour());
                    }
                }
            }
            new_meso_map.layer_images = MesoLayerImages {
                continentalness: DynamicImage::from(continentalness_img).to_rgba8().into(),
                erosion: Default::default(),
                peaks_and_valleys: Default::default(),
                temperature: DynamicImage::from(temperature_img).to_rgba8().into(),
                humidity: Default::default(),
                altitude: Default::default(),
                resources: Default::default()
            };
            new_meso_map.image = DynamicImage::from(meso_image);
            return new_meso_map;
        }).collect();

    let elapsed = now.elapsed();
    println!("Time to generate MacroMap: {:.2?}", elapsed);

    let macromap = MacroMap {
        size: (MAP_WIDTH, MAP_HEIGHT),
        meso_maps: results,
        meso_pixels: MESO_LOW_RES_PIXELS,
        generators : Generators {
            continentalness: continentalness_generator,
            temperature: temperature_generator
        }
    };

    let macro_map_entity = commands.spawn(Name::new("MacroMap")).id();
    let map_size = TilemapSize { x: (MAP_WIDTH / MESO_LOW_RES_PIXELS) as u32, y: (MAP_HEIGHT / MESO_LOW_RES_PIXELS) as u32 };

    let mut tile_storage = TileStorage::empty(map_size);
    let mut meso_map_images: Vec<Handle<Image>> = vec![];
    let mut meso_map_entites: Vec<Entity> = vec![];

    for (i, meso_map) in macromap.meso_maps.iter().enumerate() {
        let tile_pos = TilePos { x: meso_map.index.x as u32, y: map_size.y - 1 - meso_map.index.y as u32};
        let tile_entity = commands
            .spawn((TileBundle {
                position: tile_pos,
                texture_index: TileTextureIndex(i as u32),
                tilemap_id: TilemapId(macro_map_entity),
                ..Default::default()
            }, Name::new(format!("MesoMap{}",meso_map.index))))
            .id();
        meso_map_entites.push(tile_entity);
        tile_storage.set(&tile_pos, tile_entity);

        meso_map_images.push(images.add(
            Image::from_dynamic(meso_map.image.clone()
                                    .fliph(),false, RenderAssetUsages::default())
        ));
    }

    let tile_size = TilemapTileSize { x: (MESO_LOW_RES_PIXELS * DETAIL_FACTOR) as f32, y: (MESO_LOW_RES_PIXELS * DETAIL_FACTOR) as f32};
    let grid_size = tile_size.into();
    let map_type = TilemapType::Square;

    commands.entity(macro_map_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        texture: TilemapTexture::Vector(meso_map_images),
        storage: tile_storage,
        tile_size,
        transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0),
        ..Default::default()
    }).push_children(&*meso_map_entites);

}

pub fn div_rem<T: std::ops::Div<Output=T> + std::ops::Rem<Output=T> + Copy>(x: T, y: T) -> (T, T) {
    let quot = x / y;
    let rem = x % y;
    (quot, rem)
}

pub fn div_rem_usize(x: usize, y: usize) -> (usize, usize) {
    div_rem(x, y)
}

pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Startup, create_macro_map);
}
