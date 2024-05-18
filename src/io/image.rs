use image::{ImageBuffer, RgbImage};

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