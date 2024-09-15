use bevy::app::{App, Startup};
use bevy::asset::{Assets, AssetServer};
use bevy::prelude::{Camera2dBundle, Commands, default, Image, MouseButton, Res, ResMut};
use crate::engine::pancam::lib::{PanCam, PanCamPlugin};

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(PanCamPlugin)
       .add_systems(Startup, spawn_camera);
}
fn spawn_camera(
    mut commands: Commands,
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
}
