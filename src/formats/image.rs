//! JPG and PNG image writers.

use std::io::Cursor;
use std::path::Path;

use ::image::{ImageBuffer, ImageFormat, Rgb};
use anyhow::{anyhow, Context, Result};

use crate::output::write_output_bytes;
use crate::size::size_label;

pub fn write(path: &Path, target_size: u64, format: ImageFormat, force: bool) -> Result<()> {
    let mut side = ((target_size as f64 / 3.0).sqrt().ceil() as u32).clamp(16, 8192);
    loop {
        let image = ImageBuffer::from_fn(side, side, |x, y| {
            let n = x
                .wrapping_mul(73)
                .wrapping_add(y.wrapping_mul(151))
                .wrapping_add((x ^ y).wrapping_mul(31));
            Rgb([
                (n & 255) as u8,
                ((n >> 3) & 255) as u8,
                ((n >> 7) & 255) as u8,
            ])
        });
        let mut encoded = Cursor::new(Vec::new());
        image
            .write_to(&mut encoded, format)
            .with_context(|| format!("failed to encode {}", path.display()))?;
        let encoded = encoded.into_inner();

        if encoded.len() as u64 >= target_size {
            write_output_bytes(path, &encoded, force)?;
            return Ok(());
        }
        if side >= 8192 {
            return Err(anyhow!(
                "could not generate {} image at requested size {}; largest attempt was {} bytes",
                extension_for_image(format),
                size_label(target_size),
                encoded.len()
            ));
        }
        side = (side as f64 * 1.35).ceil() as u32;
    }
}

fn extension_for_image(format: ImageFormat) -> &'static str {
    match format {
        ImageFormat::Jpeg => "jpg",
        ImageFormat::Png => "png",
        _ => "image",
    }
}
