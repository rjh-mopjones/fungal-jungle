use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::log::{Level, LogPlugin};
use bevy::math::{uvec2, vec2, vec3};
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use bevy::window::close_on_esc;
use bevy_fast_tilemap::{FastTileMapPlugin, Map, MapBundleManaged};
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
        )
        .add_plugins(TilemapPlugin)
        .add_plugins(FastTileMapPlugin::default())
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
    mut materials: ResMut<Assets<Map>>,
) {
    render_terrain(false, &mut commands, asset_server, &mut materials) ;

    commands.spawn(Camera2dBundle::default()).insert(PanCam::default());

}

fn render_terrain(is_entity: bool,
                  mut commands: &mut Commands,
                  asset_server: Res<AssetServer>,
                  mut materials: &mut ResMut<Assets<Map>>
) {
    let macro_map = jungle_noise::tidal::generate_in_house_tidal_noise(MAP_WIDTH, MAP_HEIGHT, 42);
    if is_entity {
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
                        color: TileColor::from(macro_map.map[y as usize].map[x as usize].tile.normal_colour()),
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
    } else {
        let map = Map::builder(
            // Map size
            uvec2(MAP_WIDTH as u32, MAP_HEIGHT as u32),
            // Tile atlas
            asset_server.load("tile-sheet.png"),
            // Tile Size
            vec2(32., 32.),
        )
            .build_and_initialize(|m| {
                for y in 0..m.size().y {
                    for x in 0..m.size().x {
                        m.set(x, y, macro_map.map[y as usize].map[x as usize].tile.index() as u32);
                    }
                }
            });

        let mut bundle = MapBundleManaged::new(map, materials.as_mut());
        bundle.transform = Transform::default().with_translation(vec3(0., 0., 2.));
        commands.spawn(bundle);
    }
}
