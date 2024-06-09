use std::fmt::Debug;
use noise::{Fbm, MultiFractal, Perlin};
use noise::utils::{NoiseMap, NoiseMapBuilder, PlaneMapBuilder};
use crate::jungle_noise::generator::{Generator};
use crate::jungle_noise::source::Source;
use crate::macro_map::macro_map::{generate_macro_map, MacroMap};



pub fn generate_in_house_tidal_noise(width: usize, height: usize, seed: u32) -> MacroMap {
    const CONTINENT_FREQUENCY: f64 = 0.9;
    const CONTINENT_LACUNARITY: f64 = 1.70;

    // Do fbm perlin for base continent def
    let generator = Source::<3>::improved_perlin(1995).scale([0.012; 3])
        .fbm(8, CONTINENT_FREQUENCY, CONTINENT_LACUNARITY, 0.5);
    return generate_macro_map(width, height, &generator);

    // Do curve with control points for sea level
    // Do another fbm perlin for base continent def fb1 that carves out mountain range chunks
    // scale this output
    // take the min of the sea level curves and the mountain range chunks
    // clamp this so the bounds are -1.0 to 1.0
    // cache this

}

pub fn generate_tidal_noise(width: usize, height: usize, seed: u32) -> NoiseMap {

    const CONTINENT_FREQUENCY: f64 = 2.0;

    const CONTINENT_LACUNARITY: f64 = 2.208984375;

    const SEA_LEVEL: f64 = 0.0;

    const RIVER_DEPTH: f64 = 0.0234375;

    let baseContinentDef_fb0 = Fbm::<Perlin>::new(seed)
        .set_frequency(CONTINENT_FREQUENCY)
        .set_persistence(0.5)
        .set_lacunarity(CONTINENT_LACUNARITY)
        .set_octaves(8);

    // let baseContinentDef_cu = Curve::new(baseContinentDef_fb0)
    //     .add_control_point(-2.0000 + SEA_LEVEL, -1.625 + SEA_LEVEL)
    //     .add_control_point(-1.0000 + SEA_LEVEL, -1.375 + SEA_LEVEL)
    //     .add_control_point(0.0000 + SEA_LEVEL, -0.375 + SEA_LEVEL)
    //     .add_control_point(0.0625 + SEA_LEVEL, 0.125 + SEA_LEVEL)
    //     .add_control_point(0.1250 + SEA_LEVEL, 0.250 + SEA_LEVEL)
    //     .add_control_point(0.2500 + SEA_LEVEL, 1.000 + SEA_LEVEL)
    //     .add_control_point(0.5000 + SEA_LEVEL, 0.250 + SEA_LEVEL)
    //     .add_control_point(0.7500 + SEA_LEVEL, 0.250 + SEA_LEVEL)
    //     .add_control_point(1.0000 + SEA_LEVEL, 0.500 + SEA_LEVEL)
    //     .add_control_point(2.0000 + SEA_LEVEL, 0.500 + SEA_LEVEL);
    //
    // let baseContinentDef_fb1 = Fbm::<Perlin>::new(seed + 1)
    //     .set_frequency(CONTINENT_FREQUENCY * 4.34375)
    //     .set_persistence(0.5)
    //     .set_lacunarity(CONTINENT_LACUNARITY)
    //     .set_octaves(24);
    //
    // let baseContinentDef_sb = ScaleBias::new(baseContinentDef_fb1)
    //     .set_scale(0.5)
    //     .set_bias(0.1);
    //
    // let baseContinentDef_mi = Min::new(baseContinentDef_sb, baseContinentDef_cu);
    //
    // let baseContinentDef_cl = Clamp::new(baseContinentDef_mi).set_bounds(-1.0, 1.0);
    //
    // let baseContinentDef = Cache::new(baseContinentDef_cl);

    use std::time::Instant;

    let now = Instant::now();

    // let tidalLock = ScaleOnAxis::new(&scaled_conts).set_y_scale(1.5);

    let noise_map = PlaneMapBuilder::new(&baseContinentDef_fb0)
        .set_size(width, height)
        .set_x_bounds(0.0, 2.0)
        .set_y_bounds(0.0, 2.0)
        .build();

    let elapsed = now.elapsed();

    println!("Elapsed: {:.2?}", elapsed);
    return noise_map;
}
