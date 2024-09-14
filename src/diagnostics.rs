use bevy::app::App;
use bevy::diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::input::common_conditions::input_toggle_active;
use bevy::prelude::KeyCode;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::Escape)))
       .add_plugins(LogDiagnosticsPlugin::default())
       .add_plugins(FrameTimeDiagnosticsPlugin::default())
       .add_plugins(EntityCountDiagnosticsPlugin::default());
}