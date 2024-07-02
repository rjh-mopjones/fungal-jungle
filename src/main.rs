use bevy::{
    prelude::*,
};
use bevy::window::close_on_esc;

const SPRITE_SHEET_PATH: &str = "sprite-sheet.png";
const TILE_W: usize = 6;
const TILE_H: usize = 8;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_systems(Startup, setup)
        .add_systems(Update, close_on_esc)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands.spawn(Camera2dBundle::default());
    let texture = asset_server.load(SPRITE_SHEET_PATH);
    let layout = TextureAtlasLayout::from_grid(Vec2::new(TILE_W as f32, TILE_H as f32), 7, 1, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    commands.spawn((
        SpriteSheetBundle {
            texture,
            atlas: TextureAtlas {
                layout: texture_atlas_layout,
                index: 2,
            },
            transform: Transform::from_scale(Vec3::splat(6.0)),
            ..default()
        }
    ));}
