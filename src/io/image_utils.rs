use image::{ImageBuffer, RgbImage};
use noise::utils::{NoiseImage, NoiseMap};

pub fn generate_image(noise_map: Vec<Vec<f64>>, filename: &str) {
    let width = noise_map[0].len() as u32;
    let height = noise_map.len() as u32;

    let mut imgbuf: RgbImage = ImageBuffer::new(width, height);

    for (y, row) in noise_map.iter().enumerate() {
        for (x, &value) in row.iter().enumerate() {
            let pixel_value = ((value + 1.0) * 127.5) as u8; // Map [-1.0, 1.0] to [0, 255]
            imgbuf.put_pixel(x as u32, y as u32, image::Rgb([pixel_value, pixel_value, pixel_value]));
        }
    }

    imgbuf.save(filename).unwrap();
}

#[allow(dead_code)]
pub fn write_example_to_file(map: &NoiseMap, filename: &str) {
    use std::{fs, path::Path};

    let target = Path::new("example_images/").join(Path::new(filename));

    fs::create_dir_all(target.clone().parent().expect("No parent directory found."))
        .expect("Failed to create directories.");

    map.write_to_file(&target)
}

pub fn write_image_to_file(image: &NoiseImage, filename: &str) {
    use std::{fs, path::Path};

    let target = Path::new("example_images/").join(Path::new(filename));

    fs::create_dir_all(target.clone().parent().expect("No parent directory found."))
        .expect("Failed to create directories.");

    image.write_to_file(&target)
}
