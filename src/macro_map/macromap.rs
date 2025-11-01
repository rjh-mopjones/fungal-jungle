use bevy::app::{App, Plugin};
use bevy::prelude::OnEnter;
use bevy::asset::{Assets, Handle};
use bevy::core::Name;
use bevy::hierarchy::BuildChildren;
use bevy::math::Vec2;
use bevy::prelude::{Bundle, Commands, Component, default, Entity, GlobalTransform, Image, InheritedVisibility, ResMut, Transform, ViewVisibility, Visibility};
use bevy::render::render_asset::RenderAssetUsages;
use bevy_ecs_tilemap::map::{TilemapGridSize, TilemapId, TilemapRenderSettings, TilemapSize, TilemapSpacing, TilemapTexture, TilemapTileSize, TilemapType};
use bevy_ecs_tilemap::prelude::{get_tilemap_center_transform, MaterialTilemap, StandardTilemapMaterial, TileColor, TileFlip, TilePos, TilePosOld, TileStorage, TileTextureIndex, TileVisible};
use bevy_ecs_tilemap::FrustumCulling;
use image::DynamicImage;
use crate::macro_map::jungle_noise;
use crate::macro_map::tiling_strategy::TileType;
use crate::macro_map::generation::{WorldGenerator, WorldGenConfig};
use crate::macro_map::rendering::LayerImageGenerator;

#[derive(Default, Clone, Debug, Copy)]
pub struct TileLayer {
    pub(crate) continentalness: f64,
    pub(crate) erosion: f64,
    pub(crate) peaks_and_valleys: f64,
    pub(crate) temperature: f64,
    pub(crate) humidity: f64,
    pub(crate) altitude: f64,
    pub(crate) resources: f64,
    pub(crate) wind: f64
}

#[derive(Default, Clone, Debug)]
pub struct MesoLayerImages {
    pub(crate) aggregate: DynamicImage,
    pub(crate) continentalness: DynamicImage,
    pub(crate) wind: DynamicImage,
    pub(crate) erosion: DynamicImage,
    pub(crate) peaks_and_valleys: DynamicImage,
    pub(crate) temperature: DynamicImage,
    pub(crate) humidity: DynamicImage,
    pub(crate) altitude: DynamicImage,
    pub(crate) resources: DynamicImage,
}

#[derive(Component, Default, Clone, Debug)]
pub struct MacroLayerTextures {
    pub(crate) aggregate: TilemapTexture,
    // pub(crate) continentalness: TilemapTexture,
    // pub(crate) erosion: Vec<Handle<Image>>,
    // pub(crate) peaks_and_valleys: Vec<Handle<Image>>,
    pub(crate) temperature: TilemapTexture,
    // pub(crate) humidity: Vec<Handle<Image>>,
    // pub(crate) altitude: Vec<Handle<Image>>,
    // pub(crate) wind: Vec<Handle<Image>>,
    // pub(crate) resources: Vec<Handle<Image>>,
}

#[derive(Default, Clone, Debug)]
pub struct Tile {
    // Continentalness, Erosion, Peaks and Valleys, Temperature, Humidity, Wind
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

#[derive(Bundle, Debug, Default, Clone)]
pub struct MacroTilemap<M: MaterialTilemap> {
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
    pub frustum_culling: FrustumCulling,
    pub material: Handle<M>,
    pub layers: MacroLayerTextures,
    // pub meso_map: MesoMap,
    pub current_layer: CurrentLayer
}

#[derive(Bundle, Debug, Default, Clone)]
pub struct MesoTileBundle {
    pub position: TilePos,
    pub texture_index: TileTextureIndex,
    pub tilemap_id: TilemapId,
    pub visible: TileVisible,
    pub flip: TileFlip,
    pub color: TileColor,
    pub old_position: TilePosOld,
}

pub type MacroTilemapBundle = MacroTilemap<StandardTilemapMaterial>;

#[derive(Clone, Debug)]
pub struct Generators {
    pub(crate) continentalness:  jungle_noise::adapters::Fbm<3, jungle_noise::adapters::Scale<3, jungle_noise::sources::ImprovedPerlin<3>>>,
    // pub(crate) erosion: dyn jungle_noise::generator::Generator3D,
    // pub(crate) peaks_and_valleys: dyn jungle_noise::generator::Generator3D,
    pub(crate) temperature:  jungle_noise::adapters::Fbm<3, jungle_noise::adapters::Scale<3, jungle_noise::sources::ImprovedPerlin<3>>>
    // pub(crate) humidity: dyn jungle_noise::generator::Generator3D,
    // pub(crate) altitude: dyn jungle_noise::generator::Generator3D,
    // pub(crate) wind: dyn jungle_noise::generator::Generator3D,
    // pub(crate) resources: dyn jungle_noise::generator::Generator3D,
}

#[derive(Clone, Debug)]
pub struct MacroMap {
    pub(crate) size: (usize, usize),
    pub(crate) meso_pixels: usize,
    pub(crate) meso_maps: Vec<MesoMap>,
    pub(crate) generators: Generators
}

#[derive(Component, Clone, Debug, Default)]
pub struct CurrentLayer {
    pub layer: String
}

/// Bevy system that generates the MacroMap and sets up rendering
fn create_macro_map(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>
) {
    // Generate world using new pipeline
    let world_gen = WorldGenerator::with_default_config();
    let mut macromap = world_gen.generate();

    // Generate images for all MesoMaps
    let config = WorldGenConfig::default();
    let image_gen = LayerImageGenerator::new(&config);

    for meso_map in macromap.meso_maps.iter_mut() {
        meso_map.layer_images = image_gen.generate_images(meso_map);
    }

    let macro_map_entity = commands.spawn(Name::new("MacroMap")).id();
    let map_size = TilemapSize {
        x: (config.map_width / config.meso_low_res_pixels) as u32,
        y: (config.map_height / config.meso_low_res_pixels) as u32
    };

    let mut tile_storage = TileStorage::empty(map_size);
    let mut layer_images: MacroLayerTextures = Default::default();
    let mut meso_map_entites: Vec<Entity> = vec![];

    let mut aggregate_handle: Vec<Handle<Image>> = vec![];
    let mut temperature_handle: Vec<Handle<Image>> = vec![];

    for (i, meso_map) in macromap.meso_maps.iter().enumerate() {
        let tile_pos = TilePos { x: meso_map.index.x as u32, y: map_size.y - 1 - meso_map.index.y as u32};
        let tile_entity = commands
            .spawn((MesoTileBundle {
                position: tile_pos,
                texture_index: TileTextureIndex(i as u32),
                tilemap_id: TilemapId(macro_map_entity),
                ..Default::default()
            }, Name::new(format!("MesoMap{}", meso_map.index))))
            .id();
        meso_map_entites.push(tile_entity);
        tile_storage.set(&tile_pos, tile_entity);

        aggregate_handle.push(images.add(Image::from_dynamic(meso_map.layer_images.aggregate.clone().fliph(),false, RenderAssetUsages::default())));
        // layer_images.altitude.push(images.add(Image::from_dynamic(meso_map.layer_images.altitude.clone().fliph(),false, RenderAssetUsages::default())));
        // layer_images.continentalness.push(images.add(Image::from_dynamic(meso_map.layer_images.continentalness.clone().fliph(),false, RenderAssetUsages::default())));
        // layer_images.erosion.push(images.add(Image::from_dynamic(meso_map.layer_images.erosion.clone().fliph(),false, RenderAssetUsages::default())));
        temperature_handle.push(images.add(Image::from_dynamic(meso_map.layer_images.temperature.clone().fliph(),false, RenderAssetUsages::default())));
        // layer_images.humidity.push(images.add(Image::from_dynamic(meso_map.layer_images.humidity.clone().fliph(),false, RenderAssetUsages::default())));
        // layer_images.peaks_and_valleys.push(images.add(Image::from_dynamic(meso_map.layer_images.peaks_and_valleys.clone().fliph(),false, RenderAssetUsages::default())));
        // layer_images.resources.push(images.add(Image::from_dynamic(meso_map.layer_images.resources.clone().fliph(),false, RenderAssetUsages::default())));
        // layer_images.wind.push(images.add(Image::from_dynamic(meso_map.layer_images.wind.clone().fliph(),false, RenderAssetUsages::default())));
    }
    layer_images.temperature = TilemapTexture::Vector(temperature_handle);
    layer_images.aggregate= TilemapTexture::Vector(aggregate_handle);

    let tile_size = TilemapTileSize {
        x: config.meso_detail_size() as f32,
        y: config.meso_detail_size() as f32
    };
    let grid_size = tile_size.into();
    let map_type = TilemapType::Square;

    commands.entity(macro_map_entity).insert(MacroTilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        texture: layer_images.aggregate.clone(),
        layers: layer_images,
        storage: tile_storage,
        tile_size,
        transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0),
        current_layer: CurrentLayer{layer: "default".parse().unwrap()},
        ..Default::default()
    }).push_children(&*meso_map_entites);

}

use crate::modes::AppMode;

pub struct MacroMapPlugin {
    pub mode: AppMode,
}

impl Plugin for MacroMapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(self.mode), create_macro_map);
    }
}
