use noise::{Add, Cache, Clamp, Curve, Fbm, Min, MultiFractal, Perlin, RidgedMulti, ScaleBias, Seedable, Turbulence, utils};
use noise::utils::{ColorGradient, ImageRenderer, NoiseMapBuilder, PlaneMapBuilder};
use crate::macro_map::macro_map::write_macro_map_to_file;

pub mod io;
pub mod jungle_noise;
mod macro_map;

fn main() {


    println!("generating macro map");
    let macro_map = jungle_noise::tidal::generate_in_house_tidal_noise(1024, 512, 1995);
    println!("macro map generated");

    write_macro_map_to_file(macro_map, "macro-map-tidally-locked.png");
    println!("macro map written");


    println!("generating noise map");
    let noise_map = jungle_noise::tidal::generate_tidal_noise(1024, 512, 1995);
    io::image_utils::write_image_to_file(
        &ImageRenderer::new()
            .set_gradient(ColorGradient::new().build_terrain_gradient())
            .render(&noise_map),
        "tidally-locked.png",
    );

}
