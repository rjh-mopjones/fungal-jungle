use rand::prelude::SliceRandom;

// The permutation table is used to shuffle the gradient vectors.
fn generate_permutation_table() -> [usize; 512] {
    let mut p: Vec<usize> = (0..256).collect();
    let mut rng = rand::thread_rng();
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

pub fn generate(width: usize, height: usize, scale: f64) -> Vec<Vec<f64>> {
    let perm = generate_permutation_table();

    let mut noise_map = vec![vec![0.0; width]; height];

    for y in 0..height {
        for x in 0..width {
            let nx = x as f64 * scale;
            let ny = y as f64 * scale;
            noise_map[y][x] = perlin_noise(nx, ny, &perm);
        }
    }
    return noise_map;
}