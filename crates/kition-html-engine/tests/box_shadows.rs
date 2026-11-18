use kition_html_engine::{RenderOptions, Rendered, Resources, render};

fn frame(css: &str, scale: f64) -> Result<Rendered, String> {
    render(
        &format!(
            "<!doctype html><style>body{{margin:0;background:white}}#box{{position:absolute;left:80px;top:80px;width:120px;height:80px;background:#5645d4;{css}}}</style><main id=stage><div id=box></div></main>"
        ),
        Resources::new(),
        &RenderOptions {
            width: 320,
            height: 280,
            scale,
            ..Default::default()
        },
    )
}

fn pixel(image: &Rendered, x: f64, y: f64, scale: f64) -> &[u8] {
    let i = ((y * scale) as usize * image.width as usize + (x * scale) as usize) * 4;
    &image.rgba[i..i + 4]
}

#[test]
fn offsets_follow_the_element_transform_and_device_scale() -> Result<(), String> {
    for scale in [1.0, 1.5, 2.0] {
        let outset = frame("box-shadow:20px 0 0 red;transform:rotate(90deg)", scale)?;
        assert_eq!(pixel(&outset, 140.0, 190.0, scale), [255, 0, 0, 255]);
        assert_eq!(pixel(&outset, 190.0, 120.0, scale), [255, 255, 255, 255]);
        let inset = frame(
            "box-shadow:inset 20px 0 0 red;transform:rotate(90deg)",
            scale,
        )?;
        assert_eq!(pixel(&inset, 140.0, 70.0, scale), [255, 0, 0, 255]);
        assert_eq!(pixel(&inset, 110.0, 100.0, scale), [86, 69, 212, 255]);
    }
    Ok(())
}

#[test]
fn inset_shadows_paint_back_to_front_and_skip_transparent_entries() -> Result<(), String> {
    for scale in [1.0, 2.0] {
        let image = frame(
            "box-shadow:inset 10px 10px 0 red,inset 20px 20px 0 blue",
            scale,
        )?;
        assert_eq!(pixel(&image, 85.0, 120.0, scale), [255, 0, 0, 255]);
        assert_eq!(pixel(&image, 95.0, 120.0, scale), [0, 0, 255, 255]);
        assert_eq!(pixel(&image, 105.0, 120.0, scale), [86, 69, 212, 255]);
        let transparent = frame(
            "box-shadow:inset 0 0 0 transparent,inset 10px 10px 0 red",
            scale,
        )?;
        let single = frame("box-shadow:inset 10px 10px 0 red", scale)?;
        assert_eq!(transparent.rgba, single.rgba);
    }
    Ok(())
}

#[test]
fn fully_contracted_shadow_shapes_have_zero_area() -> Result<(), String> {
    for scale in [1.0, 2.0] {
        let outset = frame("box-shadow:0 0 0 -100px red", scale)?;
        assert_eq!(outset.rgba, frame("", scale)?.rgba);
        let inset = frame("box-shadow:inset 0 0 0 100px red", scale)?;
        assert_eq!(inset.rgba, frame("background:red", scale)?.rgba);
    }
    Ok(())
}

#[test]
fn spread_preserves_elliptical_corners_instead_of_averaging_them() -> Result<(), String> {
    for scale in [1.0, 2.0] {
        let image = frame(
            "border-radius:35px 10px / 12px 30px;box-shadow:0 0 0 12px red",
            scale,
        )?;
        assert_eq!(pixel(&image, 69.0, 69.0, scale), [255, 255, 255, 255]);
        assert_eq!(pixel(&image, 110.0, 70.0, scale), [255, 0, 0, 255]);
        assert_eq!(pixel(&image, 210.0, 69.0, scale), [255, 255, 255, 255]);
    }
    Ok(())
}

#[test]
fn inset_shadows_use_the_padding_edge_and_preserve_border_paint() -> Result<(), String> {
    for scale in [1.0, 2.0] {
        let image = frame(
            "border:8px solid green;border-radius:24px;box-shadow:inset 10px 10px 0 red",
            scale,
        )?;
        assert_eq!(pixel(&image, 84.0, 120.0, scale), [0, 128, 0, 255]);
        assert_eq!(pixel(&image, 90.0, 120.0, scale), [255, 0, 0, 255]);
        assert_eq!(pixel(&image, 100.0, 120.0, scale), [86, 69, 212, 255]);
    }
    Ok(())
}

#[test]
fn gaussian_sigma_is_half_the_css_blur_radius_at_each_dpr() -> Result<(), String> {
    for scale in [1.0, 2.0] {
        let image = frame("box-shadow:0 0 10px black", scale)?;
        // Five CSS pixels beyond a straight edge is approximately one sigma.
        // A Gaussian edge has 15.87% opacity there (about 214 over white).
        let channel = pixel(&image, 205.0, 120.0, scale)[0];
        assert!((208..=226).contains(&channel), "scale {scale}: {channel}");
        let outer = pixel(&image, 215.0, 120.0, scale)[0];
        assert!(outer >= 253, "scale {scale}: {outer}");
    }
    Ok(())
}

#[test]
fn offscreen_boxes_and_ancestors_do_not_cull_visible_shadow_ink() -> Result<(), String> {
    for scale in [1.0, 2.0] {
        let image = frame("left:-140px;box-shadow:40px 0 0 red", scale)?;
        assert_eq!(pixel(&image, 10.0, 100.0, scale), [255, 0, 0, 255]);
    }
    Ok(())
}
