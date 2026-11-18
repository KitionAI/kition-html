//! Downsample the engine's opaque RGBA canvas into terminal character cells.
use kition_html_engine::Rendered;
use std::io::{self, Write};

pub fn write(
    output: &mut impl Write,
    image: &Rendered,
    columns: u32,
    max_rows: u32,
    color: bool,
) -> io::Result<()> {
    if image.width == 0
        || image.height == 0
        || columns == 0
        || max_rows == 0
        || image.rgba.len() as u64 != u64::from(image.width) * u64::from(image.height) * 4
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid preview dimensions or pixels",
        ));
    }
    let factor = (f64::from(columns) / f64::from(image.width))
        .min(f64::from(max_rows) * 2.0 / f64::from(image.height))
        .min(1.0);
    let width = (f64::from(image.width) * factor).floor().max(1.0) as u32;
    let height = (f64::from(image.height) * factor).floor().max(1.0) as u32;
    for y in (0..height).step_by(2) {
        let mut previous = None;
        for x in 0..width {
            let top = average(image, x, y, width, height);
            let bottom = if y + 1 < height {
                average(image, x, y + 1, width, height)
            } else {
                [255; 3]
            };
            if color {
                if previous != Some((top, bottom)) {
                    write!(
                        output,
                        "\x1b[38;2;{};{};{}m\x1b[48;2;{};{};{}m",
                        top[0], top[1], top[2], bottom[0], bottom[1], bottom[2]
                    )?;
                    previous = Some((top, bottom));
                }
                write!(output, "\u{2580}")?;
            } else {
                let luminance = |c: [u8; 3]| {
                    (54 * u32::from(c[0]) + 183 * u32::from(c[1]) + 19 * u32::from(c[2])) / 256
                };
                let brightness = (luminance(top) + luminance(bottom)) / 2;
                let index = ((255 - brightness) * 9 + 127) / 255;
                output.write_all(&b" .:-=+*#%@"[index as usize..index as usize + 1])?;
            }
        }
        if color {
            output.write_all(b"\x1b[0m")?;
        }
        output.write_all(b"\n")?;
    }
    Ok(())
}

fn average(image: &Rendered, x: u32, y: u32, width: u32, height: u32) -> [u8; 3] {
    let x0 = u64::from(x) * u64::from(image.width) / u64::from(width);
    let x1 = u64::from(x + 1) * u64::from(image.width) / u64::from(width);
    let y0 = u64::from(y) * u64::from(image.height) / u64::from(height);
    let y1 = u64::from(y + 1) * u64::from(image.height) / u64::from(height);
    let mut sum = [0_u64; 3];
    for y in y0..y1 {
        for x in x0..x1 {
            let offset = ((y * u64::from(image.width) + x) * 4) as usize;
            for (channel, sum) in sum.iter_mut().enumerate() {
                *sum += u64::from(image.rgba[offset + channel]);
            }
        }
    }
    let area = (x1 - x0) * (y1 - y0);
    sum.map(|v| ((v + area / 2) / area) as u8)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn image(width: u32, height: u32, rgba: Vec<u8>) -> Rendered {
        Rendered {
            width,
            height,
            rgba,
            diagnostics: vec![],
            geometry: vec![],
        }
    }
    #[test]
    fn half_blocks_keep_the_upper_and_lower_pixel_colors_and_reset_each_row() -> io::Result<()> {
        let mut out = vec![];
        write(
            &mut out,
            &image(1, 2, vec![255, 0, 0, 255, 0, 0, 255, 255]),
            1,
            1,
            true,
        )?;
        assert_eq!(
            out,
            "\x1b[38;2;255;0;0m\x1b[48;2;0;0;255m\u{2580}\x1b[0m\n".as_bytes()
        );
        Ok(())
    }
    #[test]
    fn fitting_preserves_aspect_ratio_and_plain_output_contains_no_escapes() -> io::Result<()> {
        let mut out = vec![];
        write(
            &mut out,
            &image(8, 16, [0, 0, 0, 255].repeat(8 * 16)),
            8,
            2,
            false,
        )?;
        assert_eq!(out, b"@@\n@@\n");
        Ok(())
    }
    #[test]
    fn downsampling_averages_all_source_pixels() -> io::Result<()> {
        let mut out = vec![];
        write(
            &mut out,
            &image(
                2,
                2,
                vec![
                    0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 255,
                ],
            ),
            1,
            1,
            true,
        )?;
        assert_eq!(
            out,
            "\x1b[38;2;128;128;128m\x1b[48;2;255;255;255m\u{2580}\x1b[0m\n".as_bytes()
        );
        Ok(())
    }
}
