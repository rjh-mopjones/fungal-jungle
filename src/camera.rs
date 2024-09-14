use bevy::app::App;
use crate::engine::pancam::lib::PanCamPlugin;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(PanCamPlugin);
}
