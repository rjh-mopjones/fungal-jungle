use bevy::prelude::*;

mod macro_map;
mod engine;
mod game;
mod diagnostics;
mod camera;
mod macrosim;

fn main() {
    App::new()
        .add_plugins(game::plugin)
        .run();
}
