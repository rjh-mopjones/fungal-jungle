use bevy::prelude::LinearRgba;
use std::default::Default;
use bevy::color::palettes::css::{WHEAT, WHITE};
use bevy::diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::Name;
use bevy::input::common_conditions::input_toggle_active;
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy_ecs_tilemap::map::{TilemapId, TilemapTexture, TilemapType};
use bevy_ecs_tilemap::prelude::*;
use bevy_ecs_tilemap::{TilemapBundle, TilemapPlugin};
use bevy::prelude::Color as OtherColor;
use bevy::window::{PresentMode, WindowTheme};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use crate::engine::pancam::lib::{PanCam, PanCamPlugin};
use crate::macro_map::macro_map::{write_macro_map_to_file, write_meso_map_to_file};

pub mod jungle_noise;
mod macro_map;
mod engine;

const SPRITE_SHEET_PATH: &str = "sprite-sheet.png";
const SPRITE_SCALE_FACTOR: usize = 10;
const TILE_W: usize = 10;
const TILE_H: usize = 10;

const MAP_HEIGHT: usize = 512;
const MAP_WIDTH: usize = 1024;

const MESO_LOW_RES_PIXELS: usize = 16;
// 32 for meso map size
// 256 for micro map size but purps out at 64
const DETAIL_FACTOR: usize = 32;

#[derive(Resource)]
pub struct CursorPos(Vec2);
impl Default for CursorPos {
    fn default() -> Self {
        // Initialize the cursor pos at some far away place. It will get updated
        // correctly when the cursor moves.
        Self(Vec2::new(-1000.0, -1000.0))
    }
}

#[derive(Component)]
struct HighlightedTile;

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
            min_scale: 0.10, // prevent the camera from zooming too far in
            max_scale: Some(10.0), // prevent the camera from zooming too far out
            ..default()
        });
    render_terrain(
        &mut commands,
        asset_server,
        images
    ) ;
}

pub fn update_cursor_pos(
    camera_q: Query<(&GlobalTransform, &Camera)>,
    mut cursor_moved_events: EventReader<CursorMoved>,
    mut cursor_pos: ResMut<CursorPos>,
) {
    for cursor_moved in cursor_moved_events.read() {
        for (cam_t, cam) in camera_q.iter() {
            if let Some(pos) = cam.viewport_to_world_2d(cam_t, cursor_moved.position) {
                *cursor_pos = CursorPos(pos);
            }
        }
    }
}

fn highlight_tile(mut commands: Commands,
                  tilemap_q: Query<(
                      &TilemapSize,
                      &TilemapGridSize,
                      &TilemapType,
                      &TileStorage,
                      &Transform,
                  )>,
                  highlighted_tiles_q: Query<Entity, With<HighlightedTile>>,
                  cursor_pos: Res<CursorPos>,
) {
    for highlighted_tile_entity in highlighted_tiles_q.iter() {
        commands.entity(highlighted_tile_entity).insert(TileColor(OtherColor::LinearRgba(LinearRgba::from(WHITE))));
    }

    for (map_size, grid_size, map_type, tile_storage, map_transform) in tilemap_q.iter() {
        let cursor_pos: Vec2 = cursor_pos.0;
        let cursor_in_map_pos: Vec2 = {
            let cursor_pos = Vec4::from((cursor_pos, 0.0, 1.0));
            let cursor_in_map_pos = map_transform.compute_matrix().inverse() * cursor_pos;
            cursor_in_map_pos.xy()
        };
        if let Some(tile_pos) =
            TilePos::from_world_pos(&cursor_in_map_pos, map_size, grid_size, map_type)
        {
            if let Some(tile_entity) = tile_storage.get(&tile_pos) {
                commands.entity(tile_entity).insert(TileColor(OtherColor::LinearRgba(LinearRgba::from(WHEAT))));
                commands.entity(tile_entity).insert(HighlightedTile);
            }

        }

    }
}

fn render_terrain(mut commands: &mut Commands,
                  asset_server: Res<AssetServer>,
                  mut images: ResMut<Assets<Image>>
) {
    let macro_map = jungle_noise::tidal::generate_in_house_tidal_noise(MAP_WIDTH, MAP_HEIGHT, DETAIL_FACTOR, 42);
    write_meso_map_to_file(macro_map.clone(), 10, "meso-map.png");
    // write_macro_map_to_file(macro_map.clone(), "macro-map.png");
    let macro_map_entity = commands.spawn(Name::new("MacroMap")).id();
    let map_size = TilemapSize { x: (MAP_WIDTH / MESO_LOW_RES_PIXELS) as u32, y: (MAP_HEIGHT / MESO_LOW_RES_PIXELS) as u32 };

    let mut tile_storage = TileStorage::empty(map_size);
    let mut meso_map_images: Vec<Handle<Image>> = vec![];
    let mut meso_map_entites: Vec<Entity> = vec![];

    for (i, meso_map) in macro_map.meso_maps.iter().enumerate() {
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
            Image::from_dynamic(meso_map.low_res_dynamic_image.clone()
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

fn count_entities(world: &mut World) {
    println!("Entites {}", world.entities().total_count());
}

// create macromap camera where we reach threshold where generate meso, and slap a toggle on them
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Fungal Jungle".into(),
                name: Some("bevy.app".into()),
                resolution: (2048., 1024.).into(),
                present_mode: PresentMode::AutoVsync,
                fit_canvas_to_parent: true,
                prevent_default_event_handling: false,
                window_theme: Some(WindowTheme::Dark),
                enabled_buttons: bevy::window::EnabledButtons {
                    maximize: true,
                    ..Default::default()
                },
                visible: true,
                ..default()
            }),
            ..default()
        }).set(ImagePlugin::default_nearest()))
        .add_plugins(PanCamPlugin)
        .add_plugins(WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::Escape)))
        .add_plugins(TilemapPlugin)
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(EntityCountDiagnosticsPlugin::default())
        .init_resource::<CursorPos>()
        .add_systems(Startup, setup)
        .add_systems(First, update_cursor_pos)
        .add_systems(Update, highlight_tile)
        .run();
}
