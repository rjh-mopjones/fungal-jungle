use bevy::math::vec3;
use bevy::prelude::*;
use bevy::prelude::Color::Rgba;
use bevy::window::close_on_esc;
use bevy_pancam::{PanCam, PanCamPlugin};
use crate::macro_map::macro_map::write_macro_map_to_file;

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
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(PanCamPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, close_on_esc)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands.spawn(Camera2dBundle::default()).insert(PanCam::default());
    // let texture = asset_server.load(SPRITE_SHEET_PATH);
    // let layout =
    //     TextureAtlasLayout::from_grid(Vec2::new(TILE_W as f32, TILE_H as f32), 7, 1, None, None);
    // let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let image_macro_map = jungle_noise::tidal::generate_in_house_tidal_noise(MAP_WIDTH, MAP_HEIGHT, 42);
    write_macro_map_to_file(image_macro_map, "macro-map-tidally-locked.png");
    let macro_map = jungle_noise::tidal::generate_in_house_tidal_noise(MAP_WIDTH, MAP_HEIGHT, 42);

    for line in macro_map.map {
        for tile in line.map {
            let (x, y) = grid_to_world(tile.coords.0 as f32, tile.coords.1 as f32);
            commands.spawn(
                (SpriteBundle{
                    sprite: Sprite{
                        color: tile.tile.colour(),
                        flip_x: false,
                        flip_y: false,
                        custom_size: Option::from(Vec2::new(1.0, 1.0)),
                        rect: None,
                        anchor: Default::default(),
                    },
                    transform: Transform::from_scale(Vec3::splat(SPRITE_SCALE_FACTOR as f32))
                        .with_translation(vec3(x, y, 0.0)),
                    ..default()
                }),
            );
        }
    }
    fn grid_to_world(x : f32, y :f32) -> (f32, f32) {
        (
            x * TILE_W as f32,
            y * TILE_H as f32
        )
    }
}
