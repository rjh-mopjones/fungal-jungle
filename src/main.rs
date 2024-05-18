pub mod noise;
pub mod io;

fn main() {
    let width = 512;
    let height = 512;
    let scale = 0.1;
    let filename = "perlin_noise.png";

    let noise_map = noise::perlin::generate(width,height,scale);
    io::image::generate_image(noise_map, filename);
}
