use bevy::app::{App, Update};
use bevy::color::LinearRgba;
use bevy::color::palettes::basic::WHITE;
use bevy::color::palettes::css::WHEAT;
use bevy::input::ButtonInput;
use bevy::math::{Vec2, Vec4};
use bevy::prelude::{Camera, Commands, Component, CursorMoved, Entity, EventReader, GlobalTransform, KeyCode, Query, Res, ResMut, Resource, Transform, Vec4Swizzles, With};
use bevy::prelude::Color as OtherColor;
use bevy_ecs_tilemap::map::{TilemapGridSize, TilemapSize, TilemapTexture, TilemapType};
use bevy_ecs_tilemap::prelude::{TileColor, TilePos, TileStorage};
use bevy_ecs_tilemap::{TilemapPlugin};
use crate::macro_map::macromap::{CurrentLayer, MacroLayerTextures};

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
fn switch_layer(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut CurrentLayer, &mut TilemapTexture, &mut MacroLayerTextures)>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        for (mut current_layer, mut tilemap_texture, macrolayer_textures) in &mut query {
            if current_layer.layer == "default" {
                current_layer.layer = "temperature".parse().unwrap();
                println!("{}", current_layer.layer);
                *tilemap_texture = macrolayer_textures.temperature.clone();
            } else{
                current_layer.layer = "default".parse().unwrap();
                println!("{}", current_layer.layer);
                *tilemap_texture = macrolayer_textures.aggregate.clone();
            }
        }
    }
}

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(TilemapPlugin)
       .add_plugins(crate::macro_map::macromap::plugin)
       .init_resource::<CursorPos>()
       .add_systems(Update, update_cursor_pos)
       .add_systems(Update, highlight_tile)
       .add_systems(Update, switch_layer);
}
