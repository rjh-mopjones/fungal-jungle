use std::time::Instant;
use noise::{MultiFractal, Seedable};
use noise::utils::{NoiseMapBuilder};
use crate::macro_map::macro_map::write_macro_map_to_file;

pub mod io;
pub mod jungle_noise;
mod macro_map;

fn main() {
    let start = Instant::now();
    let macro_map = jungle_noise::tidal::generate_in_house_tidal_noise(1024, 512, 42);
    let duration = start.elapsed();
    println!("Time elapsed: {:?}", duration);
    write_macro_map_to_file(macro_map, "macro-map-tidally-locked.png");
}
