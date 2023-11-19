// ======================== Image processing ========================

use image::{ImageBuffer, Rgb, Rgba};

use crate::ShadeMethod;

pub const SHADE_METHOD: &[(ShadeMethod, &str)] = &[
    (ShadeMethod::Ascii, " .-:=+*#%@"),
    (ShadeMethod::Blocks, " ░▒▓█"),
    (ShadeMethod::Half, " ▄▀█"),
    (ShadeMethod::Custom(None), "your characters here"),
];

pub fn shade(pixel: Rgba<u8>, shade_method: &ShadeMethod) -> char {
    let shade_ascii = |shade_map: &str| {
        let gray = grayscale_value(pixel);
        shade_map
            .chars()
            .nth((gray as f32 / 255.0 * (shade_map.len() as f32)) as usize)
            .unwrap_or(shade_map.chars().last().unwrap())
    };
    match shade_method {
        ShadeMethod::Ascii => shade_ascii(SHADE_METHOD[0].1),
        ShadeMethod::Blocks => shade_ascii(SHADE_METHOD[1].1),
        ShadeMethod::Custom(shade_map) => shade_ascii(shade_map.as_ref().unwrap()),
        _ => panic!("Invalid shade method for single pixel"),
    }
}

pub fn invert(pixel: Rgba<u8>) -> Rgba<u8> {
    Rgba([255 - pixel[0], 255 - pixel[1], 255 - pixel[2], pixel[3]])
}

pub fn invert_img(img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>) {
    for pixel in img.pixels_mut() {
        *pixel = invert(*pixel);
    }
}

pub fn grayscale_value(pixel: Rgba<u8>) -> u8 {
    (pixel[0] as f32 * 0.2126 + pixel[1] as f32 * 0.7152 + pixel[2] as f32 * 0.0722).round() as u8
}

pub fn grayscale(pixel: Rgba<u8>) -> Rgba<u8> {
    // TODO: Use a better grayscale algorithm
    let gray = grayscale_value(pixel);
    Rgba([gray, gray, gray, pixel[3]])
}

pub fn grayscale_img(img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>) {
    for pixel in img.pixels_mut() {
        *pixel = grayscale(*pixel);
    }
}

pub fn brightness_img(img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, value: i32) {
    image::imageops::brighten(img, value);
}

pub fn contrast_img(img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, value: f32) {
    image::imageops::contrast(img, value);
}

pub fn hue_rotate_img(img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, value: i32) {
    image::imageops::huerotate(img, value);
}

pub fn rgba_to_rgb(p: Rgba<u8>) -> Rgb<u8> {
    let a = p[3] as f32 / 255.0;
    Rgb([
        (p[0] as f32 * a) as u8,
        (p[1] as f32 * a) as u8,
        (p[2] as f32 * a) as u8,
    ])
}

pub fn color_distance(a: Rgb<u8>, b: Rgb<u8>) -> f32 {
    let a = [a[0] as f32, a[1] as f32, a[2] as f32];
    let b = [b[0] as f32, b[1] as f32, b[2] as f32];
    let dist = (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2);
    dist.sqrt()
}

pub const TRANSPARENT: Rgba<u8> = Rgba([0, 0, 0, 0]);
pub fn is_transparent(pixel: Rgba<u8>) -> bool {
    pixel[3] == TRANSPARENT[3]
}

/// Remove a color from the background of an image
pub fn remove_bg_color(
    img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    rm_bg_color: Rgb<u8>,
    rm_tolerance: f32,
) {
    for pixel in img.pixels_mut() {
        if color_distance(rgba_to_rgb(*pixel), rm_bg_color) < rm_tolerance {
            *pixel = TRANSPARENT;
        }
    }
}
