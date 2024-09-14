use bevy::app::{App, Startup, Update};
use bevy::asset::{Assets, AssetServer, Handle};
use bevy::color::LinearRgba;
use bevy::color::palettes::basic::WHITE;
use bevy::color::palettes::css::WHEAT;
use bevy::core::Name;
use bevy::math::{Vec2, Vec4};
use bevy::prelude::{BuildChildren, Camera, Camera2dBundle, Commands, Component, CursorMoved, default, Entity, EventReader, GlobalTransform, Image, MouseButton, Query, Res, ResMut, Resource, Transform, Vec4Swizzles, With};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::prelude::Color as OtherColor;
use bevy::sprite::SliceScaleMode::Tile;
use bevy_ecs_tilemap::map::{TilemapGridSize, TilemapId, TilemapSize, TilemapTexture, TilemapTileSize, TilemapType};
use bevy_ecs_tilemap::prelude::{get_tilemap_center_transform, TileBundle, TileColor, TilePos, TileStorage, TileTextureIndex};
use bevy_ecs_tilemap::{TilemapBundle, TilemapPlugin};
use crate::engine::pancam::lib::{PanCam, PanCamPlugin};

use crate::macro_map::macro_map::{generate_macro_map, write_meso_map_to_file};

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

fn render_macro_map(
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
            max_scale: Some(25.0), // prevent the camera from zooming too far out
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
    let macro_map = generate_macro_map(MAP_WIDTH, MAP_HEIGHT, DETAIL_FACTOR, 42, false);
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

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(TilemapPlugin)
        .add_systems(Startup, render_macro_map)
        .init_resource::<CursorPos>()
        .add_systems(Update, update_cursor_pos)
        .add_systems(Update, highlight_tile);
}
