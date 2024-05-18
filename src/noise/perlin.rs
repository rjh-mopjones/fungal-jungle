use rand::prelude::SliceRandom;
use rand::SeedableRng;

// The permutation table is used to shuffle the gradient vectors based on a seed
fn generate_permutation_table(seed: u64) -> [usize; 512] {
    let mut p: Vec<usize> = (0..256).collect();
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    p.shuffle(&mut rng);

    let mut perm = [0; 512];
    for i in 0..512 {
        perm[i] = p[i % 256];
    }
    perm
}

// This function will return a gradient vector based on the hashed value.
fn gradient(hash: usize, x: f64, y: f64) -> f64 {
    match hash & 3 {
        0 => x + y,
        1 => -x + y,
        2 => x - y,
        _ => -x - y,
    }
}

// A simple linear interpolation function.
fn linear_interpolation(t: f64, a: f64, b: f64) -> f64 {
    a + t * (b - a)
}

// The fade function smooths the interpolation.
fn fade(t: f64) -> f64 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

// Combine all the components to generate Perlin noise.
fn perlin_noise(x: f64, y: f64, perm: &[usize; 512]) -> f64 {
    let xi = x.floor() as usize & 255;
    let yi = y.floor() as usize & 255;
    let xf = x - x.floor();
    let yf = y - y.floor();
    let u = fade(xf);
    let v = fade(yf);

    let aa = perm[perm[xi] + yi];
    let ab = perm[perm[xi] + yi + 1];
    let ba = perm[perm[xi + 1] + yi];
    let bb = perm[perm[xi + 1] + yi + 1];

    let x1 = linear_interpolation(u, gradient(aa, xf, yf), gradient(ba, xf - 1.0, yf));
    let x2 = linear_interpolation(u, gradient(ab, xf, yf - 1.0), gradient(bb, xf - 1.0, yf - 1.0));
    linear_interpolation(v, x1, x2)
}

pub fn generate(width: u32, height: u32, scale: f64, seed: u64, octaves: usize, frequency: f64, amplitude: f64, shift: f64) -> Vec<Vec<f64>> {
    let perm = generate_permutation_table(seed);

    let mut noise_map = vec![vec![0.0; width as usize]; height as usize];

    for y in 0..height {
        for x in 0..width {
            let mut noise_value = 0.0;
            let mut freq = frequency;
            let mut amp = amplitude;

            for _ in 0..octaves {
                let nx = x as f64 * scale * freq;
                let ny = y as f64 * scale * freq;
                noise_value += perlin_noise(nx, ny, &perm) * amp;

                freq *= 2.0;
                amp *= 0.5;
            }

            noise_map[y as usize][x as usize] = noise_value-shift;
        }
    }

    noise_map
}