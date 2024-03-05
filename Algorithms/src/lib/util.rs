pub fn sample_input_from_p_zero_padding(p: Vec<Vec<i32>>, input: &Vec<Vec<Vec<f32>>>) -> Vec<f32> {
    let mut result = Vec::new();
    for i in 0..p.len() {
        let a = &p[i];
        if a[0] < 0
            || a[1] < 0
            || a[2] < 0
            || a[0] >= input.len() as i32
            || a[1] >= input[0].len() as i32
            || a[2] >= input[0][0].len() as i32
        {
            result.push(0.);
        } else {
            result.push(input[a[0] as usize][a[1] as usize][a[2] as usize]);
        }
    }
    result
}
pub fn sample_input_linear(p: Vec<Vec<i32>>, input: &Vec<Vec<f32>>) -> Vec<f32> {
    let mut result = Vec::new();
    for i in 0..p.len() {
        let a = &p[i];
        result.push(input[a[0] as usize][a[1] as usize]);
    }
    result
}
pub fn split_u128_to_u8(number: u128) -> Vec<u8> {
    let mut result = vec![0; 16];

    // Iterate over each 8-bit chunk
    for i in 0..16 {
        let shift = i * 8;
        let mut chunk = ((number >> shift) & 0xFF) as u8;
        if i == 15 && chunk >> 7 & 0b1 == 1 {
            chunk &= 0x7F;
        } // padding pos
        result[i] = chunk;
    }

    result
}
extern crate image;

use image::{DynamicImage, GenericImageView, Rgba};

fn read_and_store_image(file_path: &str) -> Option<Vec<Vec<Vec<u8>>>> {
    // Attempt to open the image file
    if let Ok(img) = image::open(file_path) {
        // Calculate the center crop region
        let (width, height) = img.dimensions();
        let size = width.min(height);
        let left = (width - size) / 2;
        let top = (height - size) / 2;
        let crop_region = (left, top, left + size, top + size);

        // Resize and center crop the image to 224x224
        let mut resized_img = img.resize_exact(224, 224, image::imageops::FilterType::Triangle);
        let cropped_img = resized_img.crop(crop_region.0, crop_region.1, crop_region.2, crop_region.3);

        // Convert the image to RGB format
        let rgb_img = cropped_img.to_rgb8();

        // Create a nested vector [3, 224, 224]
        let mut result: Vec<Vec<Vec<u8>>> = vec![vec![vec![0; 224]; 224]; 3];

        // Iterate over the pixels and store them in the nested vector
        for (y, row) in rgb_img.enumerate_rows() {
            for (x, pixel) in row.enumerate() {
                result[0][y as usize][x] = pixel.2[0];
                result[1][y as usize][x] = pixel.2[1];
                result[2][y as usize][x] = pixel.2[2];
            }
        }

        Some(result)
    } else {
        None
    }
}