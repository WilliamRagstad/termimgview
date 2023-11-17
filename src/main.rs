use std::fmt::Display;

use clap::{self, command, error::ErrorKind, CommandFactory, Parser};
use crossterm::{style, style::Color};
use image::{open, ImageBuffer, Rgb};

// <Width> / <Height> = <Font aspect ratio>
const FONT_ASPECT_RATIO: f32 = 8.0 / 17.0; // or 2.0 / 3.0;

#[derive(Debug, Clone)]
pub enum ShadeMethod {
    Ascii,
    Blocks,
    Half,
    Custom(Option<String>),
}

impl Display for ShadeMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShadeMethod::Ascii => write!(f, "ascii"),
            ShadeMethod::Blocks => write!(f, "blocks"),
            ShadeMethod::Half => write!(f, "half"),
            ShadeMethod::Custom(_) => write!(f, "custom"),
        }
    }
}

impl ShadeMethod {
    pub fn height_multiplier(&self) -> f32 {
        match self {
            ShadeMethod::Half => 2.0,
            _ => 1.0,
        }
    }
}

// ======================== CLI ========================

#[derive(Parser, Debug)]
#[command(
    name = env!("CARGO_PKG_NAME"),
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    about = format!("{}\nby {}", env!("CARGO_PKG_DESCRIPTION"), env!("CARGO_PKG_AUTHORS")),
    after_help = format!(
        "Shade methods:\n{}\n\nExample usage:\n - {} .\\tests\\1.png -s 0.15 -m \" -:!|#@@@@@@@@\"\n - {} .\\tests\\2.jpg -s 1 -i -m ascii",
        processing::SHADE_METHOD.iter().enumerate().map(|(_, (i, s))| format!(" - {}: '{}'", i, s)).collect::<Vec<String>>().join("\n"),
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_NAME")),
    arg_required_else_help = true)]
struct Cli {
    #[clap(help = "Path to the image file to be displayed")]
    file: String,
    #[clap(
        short = 'm',
        long,
        value_enum,
        default_value = "blocks",
        help = "Shading method"
    )]
    shade_method: String,
    #[clap(short, long, default_value = "1", help = "The scale of the image")]
    scale: f32,
    #[clap(short, long, default_value = "false", help = "Grayscale image?")]
    grayscale: bool,
    #[clap(short, long, default_value = "false", help = "Invert image?")]
    invert: bool,
    #[clap(short, long, default_value_t = FONT_ASPECT_RATIO, help = "Adjust aspect ratio")]
    adjust_aspect_ratio: f32,
    #[clap(
        short = 'b',
        long,
        default_value = "1",
        help = "Brightness of the image"
    )]
    brightness: f32,
    #[clap(
        short = 'r',
        long,
        default_value = "0",
        help = "Rotate the hue of the image"
    )]
    hue_rotation: f32,
    // resolution_multiplier: f32,
}

fn args() -> (Cli, ShadeMethod) {
    let args = Cli::parse();
    let shading = match args.shade_method.to_lowercase().as_str() {
        "ascii" => ShadeMethod::Ascii,
        "blocks" => ShadeMethod::Blocks,
        "half" => ShadeMethod::Half,
        mapping => {
            if !mapping.is_empty() {
                ShadeMethod::Custom(Some(mapping.to_string()))
            } else {
                Cli::command()
                    .error(
                        ErrorKind::ValueValidation,
                        &format!("Invalid shade method: {}", mapping),
                    )
                    .print()
                    .unwrap();
                std::process::exit(1);
            }
        }
    };
    (args, shading)
}

fn main() {
    let (args, shading) = args();
    let mut img = load_image(&args.file);
    if args.adjust_aspect_ratio != 1.0 || args.scale != 1.0 {
        // Stretch the image in the y direction to match the font aspect ratio
        let aspect_adjust_height = img.height() as f32 * args.adjust_aspect_ratio;
        let scaled_width = img.width() as f32 * args.scale;
        let scaled_height = aspect_adjust_height as f32 * args.scale;
        let scaled_height = scaled_height * shading.height_multiplier();
        img = image::imageops::resize(
            &img,
            scaled_width as u32,
            scaled_height as u32,
            image::imageops::FilterType::Nearest,
        );
    }
    if args.invert {
        processing::invert_img(&mut img);
    }
    if args.grayscale {
        processing::grayscale_img(&mut img);
    }
    display(&img, shading).unwrap();
}

// ======================== Utility ========================

pub fn load_image(path: &str) -> image::RgbImage {
    let img = open(path).expect("Failed to open image");
    img.to_rgb8()
}

pub fn display(
    img: &ImageBuffer<Rgb<u8>, Vec<u8>>,
    shading: ShadeMethod,
) -> Result<(), std::io::Error> {
    let mut out = std::io::stdout();
    match shading {
        ShadeMethod::Half => display_stream_half(&mut out, img),
        _ => display_stream_simple(&mut out, img, shading),
    }
}

fn image_to_crossterm_color(pixel: Rgb<u8>) -> Color {
    Color::Rgb {
        r: pixel[0],
        g: pixel[1],
        b: pixel[2],
    }
}

fn color_distance(a: Rgb<u8>, b: Rgb<u8>) -> f32 {
    let a = [a[0] as f32, a[1] as f32, a[2] as f32];
    let b = [b[0] as f32, b[1] as f32, b[2] as f32];
    let dist = (a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2);
    dist.sqrt()
}

// ======================== Image display ========================

fn print_stream(
    out: &mut dyn std::io::Write,
    chr: char,
    color: Rgb<u8>,
    bg_color: Option<Rgb<u8>>,
) -> Result<(), std::io::Error> {
    if let Some(bg_color) = bg_color {
        write!(
            out,
            "{}",
            style::SetBackgroundColor(image_to_crossterm_color(bg_color))
        )?;
    }
    write!(
        out,
        "{}{}{}",
        style::SetForegroundColor(image_to_crossterm_color(color)),
        chr,
        crossterm::style::ResetColor
    )
}

fn display_stream_simple(
    out: &mut dyn std::io::Write,
    img: &ImageBuffer<Rgb<u8>, Vec<u8>>,
    shading: ShadeMethod,
) -> Result<(), std::io::Error> {
    let (width, height) = img.dimensions();
    for y in 0..height {
        for x in 0..width {
            let pixel = *img.get_pixel(x, y);
            let chr = processing::shade(pixel, &shading);
            print_stream(out, chr, pixel, None)?;
        }
        writeln!(out)?;
    }
    Ok(())
}

/// Display the image in high resolution by performing subpixel rendering
fn display_stream_half(
    out: &mut dyn std::io::Write,
    img: &ImageBuffer<Rgb<u8>, Vec<u8>>,
) -> Result<(), std::io::Error> {
    let (width, height) = img.dimensions();
    const TRANSPARENT: Rgb<u8> = Rgb([0, 0, 0]);
    for y in 0..(height / 2) {
        for x in 0..width {
            let upper = *img.get_pixel(x, y * 2);
            let lower = *img.get_pixel(x, y * 2 + 1);
            let upper_is_transparent = color_distance(upper, TRANSPARENT) < 80.0;
            let lower_is_transparent = color_distance(lower, TRANSPARENT) < 80.0;
            if upper_is_transparent && lower_is_transparent {
                print_stream(out, ' ', upper, None)?;
            } else if color_distance(upper, lower) < 10.0 {
                print_stream(out, '█', upper, None)?;
            } else if lower_is_transparent {
                print_stream(out, '▀', upper, None)?;
            } else if upper_is_transparent {
                print_stream(out, '▄', lower, None)?;
            } else {
                print_stream(out, '▀', upper, Some(lower))?;
            }
        }
        writeln!(out)?;
    }
    Ok(())
}

// ======================== Image processing ========================

mod processing {
    use super::*;

    pub const SHADE_METHOD: &[(ShadeMethod, &str)] = &[
        (ShadeMethod::Ascii, " .-:=+*#%@"),
        (ShadeMethod::Blocks, " ░▒▓█"),
        (ShadeMethod::Half, " ▄▀"),
        (ShadeMethod::Custom(None), "your characters here"),
    ];

    pub fn shade(pixel: Rgb<u8>, shade_method: &ShadeMethod) -> char {
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

    pub fn invert(pixel: Rgb<u8>) -> Rgb<u8> {
        Rgb([255 - pixel[0], 255 - pixel[1], 255 - pixel[2]])
    }

    pub fn invert_img(img: &mut ImageBuffer<Rgb<u8>, Vec<u8>>) {
        for pixel in img.pixels_mut() {
            *pixel = invert(*pixel);
        }
    }

    pub fn grayscale_value(pixel: Rgb<u8>) -> u8 {
        (pixel[0] as f32 * 0.2126 + pixel[1] as f32 * 0.7152 + pixel[2] as f32 * 0.0722).round()
            as u8
    }

    pub fn grayscale(pixel: Rgb<u8>) -> Rgb<u8> {
        // TODO: Use a better grayscale algorithm
        let gray = grayscale_value(pixel);
        Rgb([gray, gray, gray])
    }

    pub fn grayscale_img(img: &mut ImageBuffer<Rgb<u8>, Vec<u8>>) {
        for pixel in img.pixels_mut() {
            *pixel = grayscale(*pixel);
        }
    }
}
