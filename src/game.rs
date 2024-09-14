use bevy::app::{App, PluginGroup, Startup, Update};
use bevy::asset::{Assets, AssetServer, Handle};
use bevy::color::LinearRgba;
use bevy::color::palettes::basic::WHITE;
use bevy::color::palettes::css::WHEAT;
use bevy::core::Name;
use bevy::DefaultPlugins;
use bevy::diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::input::common_conditions::input_toggle_active;
use bevy::math::{Vec2, Vec4};
use bevy::prelude::{BuildChildren, Camera, Camera2dBundle, Commands, Component, CursorMoved, default, Entity, EventReader, GlobalTransform, Image, ImagePlugin, KeyCode, MouseButton, Query, Res, ResMut, Resource, Transform, Vec4Swizzles, Window, WindowPlugin, With};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::window::{PresentMode, WindowTheme};
use bevy_ecs_tilemap::map::{TilemapGridSize, TilemapId, TilemapSize, TilemapTexture, TilemapTileSize, TilemapType};
use bevy_ecs_tilemap::prelude::{get_tilemap_center_transform, TileBundle, TileColor, TilePos, TileStorage, TileTextureIndex};
use bevy_ecs_tilemap::{TilemapBundle, TilemapPlugin};
use bevy::prelude::Color as OtherColor;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use crate::engine::pancam::lib::{PanCam, PanCamPlugin};
use crate::macro_map::macro_map::{generate_macro_map, write_meso_map_to_file};



pub(super) fn plugin(app: &mut App) {
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
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
    .add_plugins(crate::camera::plugin)
    .add_plugins(crate::diagnostics::plugin)
    .add_plugins(crate::macrosim::plugin);
}