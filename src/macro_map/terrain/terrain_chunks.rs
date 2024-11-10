use std::collections::HashMap;
use std::time::SystemTime;
use bevy::app::{App, Startup};
use bevy::asset::{Assets, Handle};
use bevy::core::Name;
use bevy::prelude::{Bundle, Commands, Component, Entity, GlobalTransform, Image, InheritedVisibility, ResMut, Transform, ViewVisibility, Visibility};
use bevy::render::render_asset::RenderAssetUsages;
use bevy_ecs_tilemap::FrustumCulling;
use bevy_ecs_tilemap::map::{TilemapGridSize, TilemapId, TilemapRenderSettings, TilemapSize, TilemapSpacing, TilemapTexture, TilemapTileSize, TilemapType};
use bevy_ecs_tilemap::prelude::{get_tilemap_center_transform, MaterialTilemap, StandardTilemapMaterial, TileColor, TileFlip, TilePos, TilePosOld, TileStorage, TileTextureIndex, TileVisible};
use crate::macro_map::terrain::noise_layers::{AltitudeStrategy, ContinentalnessStrategy, NoiseLayers, NoiseStrategies, NoiseStrategy, NoiseValues, TemperatureStrategy};
use bevy::prelude::BuildChildren;
use crate::macro_map::terrain::tiling::{TilingConfig, TilingStrategy};

#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug, Default)]
pub struct ChunkCoord {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Default, Clone, Debug)]
pub struct WorldTextures {
    pub(crate) aggregate: TilemapTexture,
    pub(crate) continentalness: TilemapTexture,
    // pub(crate) erosion: Vec<Handle<Image>>,
    // pub(crate) peaks_and_valleys: Vec<Handle<Image>>,
    pub(crate) temperature: TilemapTexture,
    // pub(crate) humidity: Vec<Handle<Image>>,
    pub(crate) altitude: TilemapTexture,
    // pub(crate) wind: Vec<Handle<Image>>,
    // pub(crate) resources: Vec<Handle<Image>>,
    aggregate_handle: Vec<Handle<Image>>,
    continentalness_handle: Vec<Handle<Image>>,
    temperature_handle: Vec<Handle<Image>>,
    altitude_handle: Vec<Handle<Image>>
}
impl WorldTextures {
    pub fn add_layer(&mut self, noise_layers: &NoiseLayers, mut images: &mut ResMut<Assets<Image>>) {
        self.aggregate_handle.push(images.add(Image::from_dynamic(noise_layers.aggregate.fliph(),false, RenderAssetUsages::default())));
        self.temperature_handle.push(images.add(Image::from_dynamic(noise_layers.aggregate.fliph(),false, RenderAssetUsages::default())));
        self.continentalness_handle.push(images.add(Image::from_dynamic(noise_layers.continentalness.fliph(),false, RenderAssetUsages::default())));
        self.altitude_handle.push(images.add(Image::from_dynamic(noise_layers.altitude.fliph(),false, RenderAssetUsages::default())));
    }

    pub fn get_texture(&mut self, layer: &str ) -> TilemapTexture {
        match layer {
            "aggregate" => TilemapTexture::Vector(self.aggregate_handle.clone()),
            "continentalness" => TilemapTexture::Vector(self.continentalness_handle.clone()),
            "temperature" => TilemapTexture::Vector(self.temperature_handle.clone()),
            "altitude" => TilemapTexture::Vector(self.altitude_handle.clone()),
            _ => TilemapTexture::Vector(self.aggregate_handle.clone()),
        }
    }

}

#[derive(Component, Default)]
pub struct MacroChunk {
    pub coord: ChunkCoord,
    pub size: usize,
    pub max_meso_chunks: usize,
    pub noise_values: Vec<NoiseValues>,
    pub noise_layers: NoiseLayers,
}

#[derive(Bundle, Default)]
pub struct MacroChunkBundle {
    pub position: TilePos,
    pub texture_index: TileTextureIndex,
    pub tilemap_id: TilemapId,
    pub visible: TileVisible,
    pub flip: TileFlip,
    pub color: TileColor,
    pub old_position: TilePosOld,
    pub macro_chunk: MacroChunk
}

impl MacroChunk {
    pub fn new(size: usize,
                coord: ChunkCoord, tiling_strategy: &TilingStrategy, noise_strategies: &NoiseStrategies) -> Self {
        let mut noise_layers = NoiseLayers::new(size);
        let mut noise_values = vec![NoiseValues::default(); size * size];

        for y in 0..size {
            for x in 0..size {
                let world_x = coord.x as f64 + x as f64 / size as f64;
                let world_y = coord.y as f64 + y as f64 / size as f64;

                let index = y * size + x;
                let current_noise_value = noise_strategies.generate(world_x, world_y, 0);
                noise_layers.add_at_index(x, y, &current_noise_value, tiling_strategy);
                noise_values[index] = current_noise_value;
            }
        }

        Self {
            coord,
            size,
            noise_values,
            noise_layers,
            max_meso_chunks: 128,
        }
    }
}

pub struct ChunkingConfig {
    pub macro_chunk_size: usize,
    pub meso_chunk_size: usize,
    pub map_width: usize,
    pub map_height: usize
}

pub type WorldTileMap = WorldTileMapBundle<StandardTilemapMaterial>;

#[derive(Bundle)]
pub struct WorldTileMapBundle<M: MaterialTilemap> {
    pub grid_size: TilemapGridSize,
    pub map_type: TilemapType,
    pub size: TilemapSize,
    pub spacing: TilemapSpacing,
    pub storage: TileStorage,
    pub texture: TilemapTexture,
    pub tile_size: TilemapTileSize,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub render_settings: TilemapRenderSettings,
    /// User indication of whether an entity is visible
    pub visibility: Visibility,
    /// Algorithmically-computed indication of whether an entity is visible and should be extracted
    /// for rendering
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
    /// User indication of whether tilemap should be frustum culled.
    pub frustum_culling: FrustumCulling ,
    pub material: Handle<M>,
    pub world_textures: WorldTextures,
}

#[derive(Component)]
pub struct WorldChunks {
    world_map_entity: Entity,
    noise_strategies: NoiseStrategies,
    tiling_strategy: TilingStrategy,
    chunking_config: ChunkingConfig,
    selected_layer: String
}

impl WorldChunks {
    pub fn new(
               mut commands: Commands,
               mut images: ResMut<Assets<Image>>,
               noise_strategies: NoiseStrategies,
               tiling_strategy: TilingStrategy,
               chunking_config: ChunkingConfig) -> Self {
        let chunks_y = chunking_config.map_height / chunking_config.macro_chunk_size;
        let chunks_x = chunking_config.map_width / chunking_config.macro_chunk_size;
        let mut world_x = 0;
        let mut world_y = 0;
        let mut macro_chunk_entities: Vec<Entity> = vec![];

        let world_chunks_entity = commands.spawn(Name::new("World_Chunks")).id();
        let mut world_textures : WorldTextures = Default::default();
        let map_size = TilemapSize { x: chunking_config.map_width as u32, y: chunking_config.map_height as u32 };
        let mut tile_storage = TileStorage::empty(map_size);

        println!("Creating {} chunks", chunks_x * chunks_y);
        for y in 0..chunks_y {
            for x in 0..chunks_x {
                world_x = x * chunking_config.macro_chunk_size;
                world_y = y * chunking_config.macro_chunk_size;
                println!("On MacroChunk {} ", x + (chunks_y * y) );
                let chunk_coord = ChunkCoord{x: (x * chunking_config.macro_chunk_size) as i32,
                                             y: (y * chunking_config.macro_chunk_size) as i32 };

                let tile_pos = TilePos { x: x as u32, y: y as u32 };
                let macro_chunk = MacroChunk::new(chunking_config.macro_chunk_size, chunk_coord, &tiling_strategy, &noise_strategies);
                world_textures.add_layer(&macro_chunk.noise_layers, &mut images);

                let macro_chunk_entity = commands.spawn((MacroChunkBundle {
                    position: TilePos { x: x as u32, y: y as u32 },
                    texture_index: TileTextureIndex((x + (chunks_y * y)) as u32),
                    tilemap_id: TilemapId(world_chunks_entity),
                    macro_chunk,
                    ..Default::default()
                }, Name::new(format!("MacroChunk(x:{},y:{})",x, y)))).id();

                macro_chunk_entities.insert(x + (y * x), macro_chunk_entity);
                tile_storage.set(&tile_pos, macro_chunk_entity);
            }
        }

        let tile_size =  TilemapTileSize { x: (chunking_config.macro_chunk_size) as f32, y: (chunking_config.macro_chunk_size) as f32} ;
        let grid_size =  tile_size.into();
        let map_type =  TilemapType::Square;

        let world_map_entity = commands.entity(world_chunks_entity).insert(WorldTileMap {
            grid_size,
            map_type,
            size: map_size,
            texture: world_textures.get_texture("aggregate"),
            world_textures,
            storage: tile_storage,
            tile_size,
            transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0),
            frustum_culling: Default::default(),
            global_transform: Default::default(),
            inherited_visibility: Default::default(),
            render_settings: Default::default(),
            visibility: Default::default(),
            view_visibility: Default::default(),
            // Add other default initializations here if needed
            material: Default::default(),  // Ensure the material has a default value
            spacing: Default::default(),
        }).push_children(&macro_chunk_entities).id();

        Self {
            world_map_entity,
            chunking_config,
            noise_strategies,
            tiling_strategy,
            selected_layer: "aggregate".to_string()
        }
    }
}

fn generate_world_chunks(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>
) {
    let x = 1;
    WorldChunks::new(commands, images,
                     NoiseStrategies {
                         continentalness_strategy: ContinentalnessStrategy::new(42),
                         temperature_strategy: TemperatureStrategy::new(42),
                         altitude_strategy: AltitudeStrategy::new(42)
                     },
                     TilingStrategy::new(
                         TilingConfig{
                             sea_level: 0.1,
                             river_threshold: 0.1,
                         }
                    ), 
                    ChunkingConfig {
                        macro_chunk_size: 32,
                        meso_chunk_size: 32,
                        map_width: 1024,
                        map_height: 512,
                    });
}


pub(crate) fn plugin(app: &mut App) {
    app.add_systems(Startup, generate_world_chunks);
}
