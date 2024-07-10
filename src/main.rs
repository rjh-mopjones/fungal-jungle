use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy_ecs_tilemap::map::{TilemapId, TilemapTexture, TilemapType};
use bevy_ecs_tilemap::prelude::{get_tilemap_center_transform, TileBundle, TilemapSize, TilemapTileSize, TilePos, TileStorage, TileTextureIndex};
use bevy_ecs_tilemap::{TilemapBundle, TilemapPlugin};
use image::imageops::FilterType;
use crate::engine::pancam::lib::{PanCam, PanCamPlugin};
use crate::macro_map::macro_map::{write_meso_map_to_file};

pub mod jungle_noise;
mod macro_map;
mod engine;

const SPRITE_SHEET_PATH: &str = "sprite-sheet.png";
const SPRITE_SCALE_FACTOR: usize = 10;
const TILE_W: usize = 10;
const TILE_H: usize = 10;

const MAP_HEIGHT: usize = 512;
const MAP_WIDTH: usize = 1024;

static mut SCROLL_LEVEL: f32 = 0.0;
//WINDOW

fn main() {
    App::new()
        .add_plugins(DefaultPlugins
             .set(ImagePlugin::default_nearest())
        )
        .add_plugins(PanCamPlugin)
        .add_plugins(TilemapPlugin)
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    images: ResMut<Assets<Image>>
) {
    commands.spawn(Camera2dBundle::default())
        .insert(PanCam {
            grab_buttons: vec![MouseButton::Left, MouseButton::Middle], // which buttons should drag the camera
            enabled: true, // when false, controls are disabled. See toggle example.
            zoom_to_cursor: true, // whether to zoom towards the mouse or the center of the screen
            min_scale: 0.25, // prevent the camera from zooming too far in
            max_scale: Some(2.5), // prevent the camera from zooming too far out
            ..default()
        });
    render_terrain(
        &mut commands,
        asset_server,
        images
    ) ;
}

fn render_terrain(mut commands: &mut Commands,
                  asset_server: Res<AssetServer>,
                  mut images: ResMut<Assets<Image>>
) {
    let macro_map = jungle_noise::tidal::generate_in_house_tidal_noise(MAP_WIDTH, MAP_HEIGHT, 42);
    write_meso_map_to_file(macro_map.clone(), 10, "meso-map.png");
    let texture_handle: Handle<Image> = asset_server.load("tiles.png");
    let tilemap_entity = commands.spawn_empty().id();
    let map_size = TilemapSize { x: (MAP_WIDTH / 16) as u32, y: (MAP_HEIGHT / 16) as u32 };

    let mut tile_storage = TileStorage::empty(map_size);
    let mut meso_map_images: Vec<Handle<Image>> = vec![];

    for (i, meso_map) in macro_map.meso_maps.iter().enumerate() {
            let tile_pos = TilePos { x: meso_map.index.x as u32, y: map_size.y - 1 - meso_map.index.y as u32};
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    texture_index: TileTextureIndex(i as u32),
                    tilemap_id: TilemapId(tilemap_entity),
                    ..Default::default()
                })
                .id();
            tile_storage.set(&tile_pos, tile_entity);
        meso_map_images.push(images.add(
            Image::from_dynamic(meso_map.low_res_dynamic_image.clone()
                                    .fliph()
                                    .resize(64, 64, FilterType::Nearest), false, RenderAssetUsages::default())
        ));
    }

    let tile_size = TilemapTileSize { x: 64.0, y: 64.0 };
    let grid_size = tile_size.into();
    let map_type = TilemapType::Square;

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        texture: TilemapTexture::Vector(meso_map_images),
        storage: tile_storage,
        tile_size,
        transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0),
        ..Default::default()
    });
}