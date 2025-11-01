use bevy::prelude::*;

mod macro_map;
mod engine;
mod game;
mod diagnostics;
mod camera;
mod macrosim;
mod modes;

fn main() {
    App::new()
        .add_plugins(game::plugin)
        .add_plugins(modes::ModesPlugin)
        .run();
}
