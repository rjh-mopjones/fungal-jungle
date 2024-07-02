use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::log::{Level, LogPlugin};
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy::window::close_on_esc;
use bevy_pancam::{PanCam, PanCamPlugin};

pub mod jungle_noise;
mod macro_map;

const SPRITE_SHEET_PATH: &str = "sprite-sheet.png";
const SPRITE_SCALE_FACTOR: usize = 10;
const TILE_W: usize = 10;
const TILE_H: usize = 10;

const MAP_HEIGHT: usize = 512;
const MAP_WIDTH: usize = 1024;

//WINDOW

fn main() {
    App::new()
        .add_plugins(DefaultPlugins
             .set(ImagePlugin::default_nearest())
             .set(LogPlugin {
                 // Everything else should be set to Level::ERROR
                 level: Level::ERROR,
                 // except for bevy_ecs_tilemap
                 filter: "bevy_ecs_tilemap=trace".into(),
                 ..default()
             }),
        )
        .add_plugins(TilemapPlugin)
        .add_plugins(PanCamPlugin)
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, close_on_esc)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn(Camera2dBundle::default()).insert(PanCam::default());
    let macro_map = jungle_noise::tidal::generate_in_house_tidal_noise(MAP_WIDTH, MAP_HEIGHT, 42);

    let texture_handle: Handle<Image> = asset_server.load("one-tile.png");
    let tilemap_entity = commands.spawn_empty().id();
    let map_size = TilemapSize { x: MAP_WIDTH as u32, y: MAP_HEIGHT as u32 };
    let mut tile_storage = TileStorage::empty(map_size);

    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let tile_pos = TilePos { x, y };
            let tile_entity = commands
                .spawn(TileBundle {
                    position: tile_pos,
                    texture_index: TileTextureIndex(0),
                    color: TileColor::from(macro_map.map[y as usize].map[x as usize].tile.colour()),
                    tilemap_id: TilemapId(tilemap_entity),
                    ..Default::default()
                })
                .id();
            tile_storage.set(&tile_pos, tile_entity);
        }
    }

    let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
    let grid_size = tile_size.into();
    let map_type = TilemapType::Square;

    commands.entity(tilemap_entity).insert(TilemapBundle {
        grid_size,
        map_type,
        size: map_size,
        texture: TilemapTexture::Single(texture_handle),
        storage: tile_storage,
        tile_size,
        transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0),
        ..Default::default()
    });
}
