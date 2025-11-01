use bevy::app::{App, PluginGroup};
use bevy::DefaultPlugins;
use bevy::prelude::{default, ImagePlugin, Window, WindowPlugin};
use bevy::window::{PresentMode, WindowTheme};

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
    .add_plugins(crate::diagnostics::plugin)
    .add_plugins(crate::camera::plugin);
}