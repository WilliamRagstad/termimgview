use std::fmt::Display;

use crossterm::style::{self, Color};
use image::{ImageBuffer, Rgb, Rgba};

use crate::{
    processing::{self, color_distance, is_transparent, rgba_to_rgb},
    ShadeMethod,
};

pub fn display(
    img: &ImageBuffer<Rgba<u8>, Vec<u8>>,
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
        style::ResetColor
    )
}

struct LineRenderer {
    buffer: Vec<char>,
    current_color: Option<Rgb<u8>>,
    current_bg_color: Option<Rgb<u8>>,
}

impl LineRenderer {
    fn new() -> Self {
        Self {
            buffer: Vec::new(),
            current_color: None,
            current_bg_color: None,
        }
    }

    fn clear(&mut self) {
        self.buffer.clear();
        self.current_color = None;
        self.current_bg_color = None;
    }

    fn display(&mut self, item: &dyn Display) {
        self.buffer
            .append(format!("{}", item).chars().collect::<Vec<char>>().as_mut());
    }

    fn add(&mut self, chr: char, color: Option<Rgb<u8>>, bg_color: Option<Rgb<u8>>) {
        if (self.current_color != color || self.current_bg_color != bg_color)
            && !self.buffer.is_empty()
        {
            self.display(&style::ResetColor);
        }
        if color.is_some() && self.current_color != color {
            self.display(&style::SetForegroundColor(image_to_crossterm_color(
                color.unwrap(),
            )));
            self.current_color = color;
        }
        if bg_color.is_some() && self.current_bg_color != bg_color {
            self.display(&style::SetBackgroundColor(image_to_crossterm_color(
                bg_color.unwrap(),
            )));
            self.current_bg_color = bg_color;
        }
        self.buffer.push(chr);
    }

    fn build(&mut self) -> String {
        self.display(&style::ResetColor);
        self.buffer.iter().collect()
    }
}

fn display_stream_simple(
    out: &mut dyn std::io::Write,
    img: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    shading: ShadeMethod,
) -> Result<(), std::io::Error> {
    let (width, height) = img.dimensions();
    let mut renderer = LineRenderer::new();
    for y in 0..height {
        for x in 0..width {
            let pixel = *img.get_pixel(x, y);
            let chr = processing::shade(pixel, &shading);
            // print_stream(out, chr, rgba_to_rgb(pixel), None)?;
            renderer.add(chr, Some(rgba_to_rgb(pixel)), None);
        }
        // writeln!(out)?;
        writeln!(out, "{}", renderer.build())?;
        renderer.clear();
    }
    Ok(())
}

/// Display the image in high resolution by performing subpixel rendering
fn display_stream_half(
    out: &mut dyn std::io::Write,
    img: &ImageBuffer<Rgba<u8>, Vec<u8>>,
) -> Result<(), std::io::Error> {
    let (width, height) = img.dimensions();
    let mut renderer = LineRenderer::new();
    for y in 0..(height / 2) {
        for x in 0..width {
            let upper = *img.get_pixel(x, y * 2);
            let lower = *img.get_pixel(x, y * 2 + 1);
            let upper_is_transparent = is_transparent(upper);
            let lower_is_transparent = is_transparent(lower);
            let upper = rgba_to_rgb(upper);
            let lower = rgba_to_rgb(lower);
            let (chr, color, bg_color) = {
                if upper_is_transparent && lower_is_transparent {
                    (' ', upper, None)
                } else if color_distance(upper, lower) < 10.0 {
                    ('█', upper, None)
                } else if lower_is_transparent {
                    ('▀', upper, None)
                } else if upper_is_transparent {
                    ('▄', lower, None)
                } else {
                    ('▀', upper, Some(lower))
                }
            };
            // print_stream(out, chr, color, bg_color)?;
            renderer.add(chr, Some(color), bg_color);
        }
        // writeln!(out)?;
        writeln!(out, "{}", renderer.build())?;
        renderer.clear();
    }
    Ok(())
}
