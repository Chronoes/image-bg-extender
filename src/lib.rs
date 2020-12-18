use serde::Deserialize;

use image::io::Reader as ImageReader;
use image::{imageops, DynamicImage, GenericImage, GenericImageView};

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ImageInfo {
    source: String,
    destination: String,
    aspect_ratio: (u32, u32),
}

#[derive(Copy, Clone)]
pub enum Orientation {
    Landscape,
    Portrait,
}

fn div(base: u32, modulo: u32) -> (u32, u32) {
    (base / modulo, base % modulo)
}

fn calculate_edge_length(length: u32) -> u32 {
    (length as f32 * 0.05).floor() as u32
}

fn average_color(img: DynamicImage) -> image::Rgba<u8> {
    let resized = img.resize_exact(1, 1, imageops::FilterType::Nearest);
    resized.get_pixel(0, 0)
}

fn aggregate_edge_colors(
    base_img: &DynamicImage,
    orientation: Orientation,
) -> (image::Rgba<u8>, image::Rgba<u8>) {
    let (width, height) = base_img.dimensions();
    let first_edge;
    let second_edge;
    if let Orientation::Landscape = orientation {
        // Image is wider than the desired aspect ratio
        let edge_length = calculate_edge_length(height);
        first_edge = average_color(base_img.crop_imm(0, 0, width, edge_length));
        second_edge = average_color(base_img.crop_imm(0, height - edge_length, width, edge_length));
    } else {
        // Image is taller than the desired aspect ratio
        let edge_length = calculate_edge_length(width);
        first_edge = average_color(base_img.crop_imm(0, 0, edge_length, height));
        second_edge = average_color(base_img.crop_imm(width - edge_length, 0, edge_length, height));
    }

    (first_edge, second_edge)
}

fn normalise_image(
    img: &DynamicImage,
    (width_overflow, height_overflow): (u32, u32),
) -> DynamicImage {
    let (width, height) = img.dimensions();
    img.crop_imm(
        width_overflow / 2,
        height_overflow / 2,
        width - width_overflow,
        height - height_overflow,
    )
}

fn calculate_canvas_dimensions(
    (aspect_width, aspect_height): (u32, u32),
    (width_multiplier, height_multiplier): (u32, u32),
    orientation: Orientation,
) -> (u32, u32) {
    return if let Orientation::Landscape = orientation {
        (
            aspect_width * width_multiplier,
            aspect_height * width_multiplier,
        )
    } else {
        (
            aspect_width * height_multiplier,
            aspect_height * height_multiplier,
        )
    };
}

fn create_split_background(
    canvas: &mut image::RgbaImage,
    first_color: image::Rgba<u8>,
    second_color: image::Rgba<u8>,
    orientation: Orientation,
) {
    let (width, height) = canvas.dimensions();

    if let Orientation::Landscape = orientation {
        for y in 0..height {
            let color = if y > height / 2 {
                first_color
            } else {
                second_color
            };
            for x in 0..width {
                canvas.put_pixel(x, y, color);
            }
        }
    } else {
        for x in 0..width {
            let color = if x < width / 2 {
                first_color
            } else {
                second_color
            };
            for y in 0..height {
                canvas.put_pixel(x, y, color);
            }
        }
    }
}

pub fn compile_image<'a>(info: &'a ImageInfo) -> Result<&'a str, Box<dyn std::error::Error>> {
    let src = info.source.as_str();
    let dest = info.destination.as_str();
    let img = ImageReader::open(src)?.decode()?;

    let (width, height) = img.dimensions();

    let (width_multiplier, width_overflow) = div(width, info.aspect_ratio.0);
    let (height_multiplier, height_overflow) = div(height, info.aspect_ratio.1);
    // Check if image is exactly within aspect ratio
    if !(width_overflow == 0 && height_overflow == 0 && width_multiplier == height_multiplier) {
        let orientation = if width_multiplier > height_multiplier {
            Orientation::Landscape
        } else {
            Orientation::Portrait
        };

        let img = normalise_image(&img, (width_overflow, height_overflow));
        let (first_edge, second_edge) = aggregate_edge_colors(&img, orientation);

        let (canvas_width, canvas_height) = calculate_canvas_dimensions(
            info.aspect_ratio,
            (width_multiplier, height_multiplier),
            orientation,
        );

        let (width, height) = img.dimensions();
        let new_img = {
            let mut bg_img = image::RgbaImage::new(canvas_width, canvas_height);
            create_split_background(&mut bg_img, first_edge, second_edge, orientation);
            bg_img.copy_from(
                &img,
                std::cmp::max((canvas_width).saturating_sub(width) / 2, 0),
                std::cmp::max((canvas_height).saturating_sub(height) / 2, 0),
            )?;
            bg_img
        };

        new_img.save(dest)?;
    } else {
        std::fs::copy(src, dest)?;
    }

    Ok(dest)
}
