use noise::{Add, Cache, Clamp, Curve, Fbm, Min, MultiFractal, Perlin, RidgedMulti, ScaleBias, ScalePoint, Seedable, Turbulence};
use noise::utils::{NoiseMap, NoiseMapBuilder, PlaneMapBuilder};
use crate::noise_wrapper::scale_on_axis::ScaleOnAxis;

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

    let baseContinentDef_cu = Curve::new(baseContinentDef_fb0)
        .add_control_point(-2.0000 + SEA_LEVEL, -1.625 + SEA_LEVEL)
        .add_control_point(-1.0000 + SEA_LEVEL, -1.375 + SEA_LEVEL)
        .add_control_point(0.0000 + SEA_LEVEL, -0.375 + SEA_LEVEL)
        .add_control_point(0.0625 + SEA_LEVEL, 0.125 + SEA_LEVEL)
        .add_control_point(0.1250 + SEA_LEVEL, 0.250 + SEA_LEVEL)
        .add_control_point(0.2500 + SEA_LEVEL, 1.000 + SEA_LEVEL)
        .add_control_point(0.5000 + SEA_LEVEL, 0.250 + SEA_LEVEL)
        .add_control_point(0.7500 + SEA_LEVEL, 0.250 + SEA_LEVEL)
        .add_control_point(1.0000 + SEA_LEVEL, 0.500 + SEA_LEVEL)
        .add_control_point(2.0000 + SEA_LEVEL, 0.500 + SEA_LEVEL);

    let baseContinentDef_fb1 = Fbm::<Perlin>::new(seed + 1)
        .set_frequency(CONTINENT_FREQUENCY * 4.34375)
        .set_persistence(0.5)
        .set_lacunarity(CONTINENT_LACUNARITY)
        .set_octaves(24);

    let baseContinentDef_sb = ScaleBias::new(baseContinentDef_fb1)
        .set_scale(0.5)
        .set_bias(0.1);

    let baseContinentDef_mi = Min::new(baseContinentDef_sb, baseContinentDef_cu);

    let baseContinentDef_cl = Clamp::new(baseContinentDef_mi).set_bounds(-1.0, 1.0);

    let baseContinentDef = Cache::new(baseContinentDef_cl);

    let riverPositions_rm0 = RidgedMulti::<Perlin>::new(seed + 100)
        .set_frequency(18.75)
        .set_lacunarity(CONTINENT_LACUNARITY)
        .set_octaves(1);

    let riverPositions_cu0 = Curve::new(riverPositions_rm0)
        .add_control_point(-2.000, 2.000)
        .add_control_point(-1.000, 1.000)
        .add_control_point(-0.125, 0.875)
        .add_control_point(0.000, -1.000)
        .add_control_point(1.000, -1.500)
        .add_control_point(2.000, -2.000);

    let riverPositions_rm1 = RidgedMulti::<Perlin>::new(seed + 101)
        .set_frequency(43.25)
        .set_lacunarity(CONTINENT_LACUNARITY)
        .set_octaves(1);

    let riverPositions_cu1 = Curve::new(riverPositions_rm1)
        .add_control_point(-2.000, 2.0000)
        .add_control_point(-1.000, 1.5000)
        .add_control_point(-0.125, 1.4375)
        .add_control_point(0.000, 0.5000)
        .add_control_point(1.000, 0.2500)
        .add_control_point(2.000, 0.0000);

    let riverPositions_mi = Min::new(riverPositions_cu0, riverPositions_cu1);

    let riverPositions_tu = Turbulence::<_, Perlin>::new(riverPositions_mi)
        .set_seed(seed + 102)
        .set_frequency(9.25)
        .set_power(1.0 / 57.75)
        .set_roughness(6);

    let riverPositions = Cache::new(riverPositions_tu);

    let continentsWithRivers_sb = ScaleBias::new(riverPositions)
        .set_scale(RIVER_DEPTH / 2.0)
        .set_bias(-RIVER_DEPTH / 2.0);

    let continentsWithRivers_ad = Add::new(&baseContinentDef, continentsWithRivers_sb);

    let scaled_conts= ScalePoint::new(&continentsWithRivers_ad).set_z_scale(1.5);
    use std::time::Instant;

    let now = Instant::now();

    let tidalLock = ScaleOnAxis::new(&scaled_conts).set_y_scale(1.5);

    let noise_map = PlaneMapBuilder::new(&tidalLock)
        .set_size(width, height)
        .set_x_bounds(0.0, 2.0)
        .set_y_bounds(0.0, 2.0)
        .build();

    let elapsed = now.elapsed();

    println!("Elapsed: {:.2?}", elapsed);
    return noise_map;
}
