use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::math::{uvec2};
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use engine::ecs_tilemap::lib::prelude::*;
use bevy::window::{close_on_esc};
use image::imageops::FilterType;
use crate::engine::ecs_tilemap::lib::TilemapBundle;
use crate::engine::fast_tilemap::map::Map;
use crate::engine::fast_tilemap::plugin::FastTileMapPlugin;
use crate::engine::pancam::lib::PanCam;
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
        .add_plugins(TilemapPlugin)
        .add_plugins(FastTileMapPlugin::default())
        .add_plugins(engine::pancam::lib::PanCamPlugin)
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, close_on_esc)
        .add_systems(Update, load_meso_map)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<Map>>,
    images: ResMut<Assets<Image>>
) {
    render_terrain(&mut commands, asset_server, &mut materials, images) ;

    commands.spawn(Camera2dBundle::default())
        .insert(PanCam {
            grab_buttons: vec![MouseButton::Left, MouseButton::Middle], // which buttons should drag the camera
            enabled: true, // when false, controls are disabled. See toggle example.
            zoom_to_cursor: true, // whether to zoom towards the mouse or the center of the screen
            min_scale: 0.25, // prevent the camera from zooming too far in
            max_scale: Some(5.), // prevent the camera from zooming too far out
            ..default()
        });

}

fn render_terrain(mut commands: &mut Commands,
                  asset_server: Res<AssetServer>,
                  mut materials: &mut ResMut<Assets<Map>>,
                  mut images: ResMut<Assets<Image>>
) {
    let macro_map = jungle_noise::tidal::generate_in_house_tidal_noise(MAP_WIDTH, MAP_HEIGHT, 42);
    write_meso_map_to_file(macro_map.clone(), 10, "meso-map.png");
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
            Image::from_dynamic(meso_map.low_res_dynamic_image.clone().fliph().resize(64, 64, FilterType::Nearest), false, RenderAssetUsages::default())
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


fn load_meso_map(
    proj: Query<(&mut OrthographicProjection)>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut camera_query: Query<(&GlobalTransform, &Camera), With<OrthographicProjection>>,
    maps: Query<&Handle<Map>>,
    mut materials: ResMut<Assets<Map>>
) {
    for event in cursor_moved_events.read() {
        for map_handle in maps.iter() {
            let map = materials.get_mut(map_handle).unwrap();

            for (global, camera) in camera_query.iter_mut() {
                // Translate viewport coordinates to world coordinates
                if let Some(world) = camera
                    .viewport_to_world(global, event.position)
                    .map(|ray| ray.origin.truncate())
                {
                    let coord = map.world_to_map(world);

                    let coord = coord
                        .as_uvec2()
                        .clamp(uvec2(0, 0), map.map_size() - uvec2(1, 1));

                    let idx = coord.y as usize * map.map_uniform.map_size.x as usize + coord.x as usize;

                    let tile = map.map_texture[idx].to_string();
                    println!("Scale: {}, Cursor Position: {}:{}, Tile: {}",  proj.single().scale,
                             coord.x, coord.y, tile
                    );
                } // if Some(world)
            } // for (global, camera)
        } // for map
    } // for event
}