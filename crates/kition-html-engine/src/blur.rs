//! CPU Gaussian approximation with unrounded intermediate box sums.
//!
//! Kernels use the three-box construction described by SVG filter effects,
//! including the identity window for subpixel sigmas in Skia's raster path.

pub fn apply(rgba: &mut [u8], width: usize, height: usize, sigma_x: f64, sigma_y: f64) {
    axis(rgba, width, height, sigma_x, false);
    axis(rgba, width, height, sigma_y, true);
}

fn axis(rgba: &mut [u8], width: usize, height: usize, sigma: f64, vertical: bool) {
    if sigma <= 0.0 {
        return;
    }
    let (length, lines, step) = if vertical {
        (height, width, width * 4)
    } else {
        (width, height, 4)
    };
    let window = (sigma * 3.0 * (2.0 * std::f64::consts::PI).sqrt() / 4.0 + 0.5)
        .floor()
        .max(1.0) as usize;
    let sizes = [
        window,
        window,
        window + usize::from(window.is_multiple_of(2)),
    ];
    let border = sizes.iter().map(|n| n - 1).sum::<usize>() / 2;
    let mut input = vec![0_u64; length + 2 * border];
    let mut output = vec![0_u64; input.len()];
    for line in 0..lines {
        let base = if vertical { line * 4 } else { line * width * 4 };
        for channel in 0..4 {
            input.fill(0);
            for position in 0..length {
                input[position + border] = u64::from(rgba[base + position * step + channel]);
            }
            for (pass, &size) in sizes.iter().enumerate() {
                let left = if pass == 1 { (size - 1) / 2 } else { size / 2 };
                let right = size - left - 1;
                let mut sum: u64 = input.iter().take(right + 1).sum();
                for (position, value) in output.iter_mut().enumerate() {
                    *value = sum;
                    if position >= left {
                        sum -= input[position - left];
                    }
                    if let Some(&next) = input.get(position + right + 1) {
                        sum += next;
                    }
                }
                std::mem::swap(&mut input, &mut output);
            }
            let divisor = sizes.iter().product::<usize>() as u64;
            for position in 0..length {
                rgba[base + position * step + channel] =
                    ((input[position + border] + divisor / 2) / divisor).min(255) as u8;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::apply;

    #[test]
    fn even_box_windows_remain_centered_without_intermediate_rounding() {
        let mut rgba = vec![0; 21 * 4];
        rgba[10 * 4..10 * 4 + 4].fill(240);
        apply(&mut rgba, 21, 1, 2.0, 0.0);
        let expected = [3, 9, 18, 30, 39, 42, 39, 30, 18, 9, 3];
        for (index, &value) in expected.iter().enumerate() {
            assert_eq!(&rgba[(index + 5) * 4..(index + 6) * 4], &[value; 4]);
        }
        assert!(rgba[..5 * 4].iter().all(|&v| v == 0));
        assert!(rgba[16 * 4..].iter().all(|&v| v == 0));
    }

    #[test]
    fn axes_are_independent_and_transpose_consistently() {
        let width = 17;
        let height = 13;
        let mut a = vec![0; width * height * 4];
        a[(6 * width + 8) * 4..(6 * width + 8) * 4 + 4].copy_from_slice(&[64, 128, 192, 255]);
        let mut b = vec![0; a.len()];
        for y in 0..height {
            for x in 0..width {
                b[(x * height + y) * 4..(x * height + y) * 4 + 4]
                    .copy_from_slice(&a[(y * width + x) * 4..(y * width + x) * 4 + 4]);
            }
        }
        apply(&mut a, width, height, 3.0, 0.0);
        apply(&mut b, height, width, 0.0, 3.0);
        for y in 0..height {
            for x in 0..width {
                assert_eq!(
                    &a[(y * width + x) * 4..(y * width + x) * 4 + 4],
                    &b[(x * height + y) * 4..(x * height + y) * 4 + 4]
                );
            }
        }
    }

    #[test]
    fn small_box_kernel_is_symmetric_and_subpixel_window_is_identity() {
        let mut rgba = vec![0; 15 * 4];
        rgba[7 * 4..8 * 4].fill(255);
        let original = rgba.clone();
        apply(&mut rgba, 15, 1, 0.0, 0.0);
        assert_eq!(rgba, original);
        apply(&mut rgba, 15, 1, 0.5, 0.0);
        assert_eq!(rgba, original);
        apply(&mut rgba, 15, 1, 1.0, 0.0);
        for x in 0..15 {
            assert_eq!(&rgba[x * 4..x * 4 + 4], &rgba[(14 - x) * 4..(15 - x) * 4]);
        }
        let total: u32 = rgba
            .as_chunks::<4>()
            .0
            .iter()
            .map(|v| u32::from(v[3]))
            .sum();
        assert!((253..=257).contains(&total));
        assert!((84..=86).contains(&rgba[7 * 4]));
    }
}
