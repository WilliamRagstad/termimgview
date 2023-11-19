use std::fmt::Display;

use clap::{self, command, error::ErrorKind, CommandFactory, Parser};
use crossterm::{style, style::Color};
use image::{open, ImageBuffer, Rgb, Rgba};

mod processing;
mod rendering;

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
    brightness: i32,
    #[clap(short = 'c', long, default_value = "1", help = "Contrast of the image")]
    contrast: f32,
    #[clap(
        short = 'u',
        long,
        default_value = "0",
        help = "Rotate the hue of the image"
    )]
    hue_rotation: i32,
    #[clap(
        short = 'r',
        long,
        default_value = "0,0,0",
        help = "Make color transparent"
    )]
    rm_color: String, // resolution_multiplier: f32,
    // Color removal tolerance
    #[clap(
        short = 't',
        long,
        default_value = "80",
        help = "Color removal tolerance"
    )]
    rm_tolerance: f32,
}

fn args() -> (Cli, ShadeMethod, Option<Rgb<u8>>) {
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
    let remove_bg_color = {
        if args.rm_color.is_empty() {
            None
        } else {
            let mut channels = args.rm_color.split(',');
            let mut get = |name: &str| {
                channels
                    .next()
                    .unwrap_or_else(|| {
                        panic!("Expected {} channel of background removal color", name)
                    })
                    .parse()
                    .unwrap()
            };
            Some(Rgb::<u8>([get("red"), get("green"), get("blue")]))
        }
    };
    (args, shading, remove_bg_color)
}

fn main() {
    let (args, shading, rm_bg_color) = args();
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
    if let Some(rm_bg_color) = rm_bg_color {
        processing::remove_bg_color(&mut img, rm_bg_color, args.rm_tolerance);
    }
    if args.brightness != 1 {
        processing::brightness_img(&mut img, args.brightness);
    }
    if args.contrast != 1.0 {
        processing::contrast_img(&mut img, args.contrast);
    }
    if args.hue_rotation != 0 {
        processing::hue_rotate_img(&mut img, args.hue_rotation);
    }
    rendering::display(&img, shading).unwrap();
}

// ======================== Utility ========================

pub fn load_image(path: &str) -> image::RgbaImage {
    let img = open(path).expect("Failed to open image");
    img.to_rgba8()
}
