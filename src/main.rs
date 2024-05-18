pub mod noise;
pub mod io;

fn main() {
     let width  = 1024;
     let height = 512;
     let seed = 41903;

    let perlin_noise_map = noise::perlin::generate(width, height, 0.02, seed,
                                            32, 0.5, 1.5 , 0.5);
    io::image::generate_image(perlin_noise_map, "perlin_noise.png");

    let worley_noise_map = noise::worley::generate(height as usize, width as usize, 20,
                                                   10, seed, 2.0, 0.5);

    io::image::generate_image(worley_noise_map, "worley_noise.png");

}
