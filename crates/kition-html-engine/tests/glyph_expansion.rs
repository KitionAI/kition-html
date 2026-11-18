use anyrender::{PaintScene, render_to_buffer};
use anyrender_vello_cpu::VelloCpuImageRenderer;
use peniko::{
    Blob, Fill, FontData,
    color::palette::css::{BLACK, WHITE},
    kurbo::{Affine, Rect, Vec2},
};
use skrifa::MetadataProvider;
use std::sync::Arc;

fn bounds(size: f32, expansion: f64, scale: f64, hint: bool) -> Result<[usize; 4], String> {
    let bytes = include_bytes!("../../../conformance/assets/JetBrainsMono-Variable.ttf");
    let reference = skrifa::FontRef::new(bytes).map_err(|e| e.to_string())?;
    let id = reference
        .charmap()
        .map('H')
        .ok_or("missing test glyph")?
        .to_u32();
    let font = FontData::new(Blob::new(Arc::new(bytes.to_vec())), 0);
    let image = render_to_buffer::<VelloCpuImageRenderer, _>(
        |scene| {
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                WHITE,
                None,
                &Rect::new(0.0, 0.0, 256.0, 256.0),
            );
            scene.draw_glyphs(
                &font,
                size,
                hint,
                &[],
                Vec2::new(expansion, expansion),
                Fill::NonZero,
                BLACK,
                1.0,
                Affine::scale(scale),
                None,
                [anyrender::Glyph {
                    id,
                    x: 24.0,
                    y: 90.0,
                }]
                .into_iter(),
            );
        },
        256,
        256,
    );
    let mut bounds = [256, 256, 0, 0];
    for y in 0..256 {
        for x in 0..256 {
            if image[(y * 256 + x) * 4] < 128 {
                bounds[0] = bounds[0].min(x);
                bounds[1] = bounds[1].min(y);
                bounds[2] = bounds[2].max(x);
                bounds[3] = bounds[3].max(y);
            }
        }
    }
    assert!(bounds[2] > bounds[0]);
    Ok(bounds)
}

#[test]
fn unhinted_expansion_uses_rendered_font_pixels_at_every_size() -> Result<(), String> {
    for size in [16.0, 32.0, 64.0] {
        let plain = bounds(size, 0.0, 1.0, false)?;
        let expanded = bounds(size, 1.0, 1.0, false)?;
        for axis in 0..2 {
            let growth = (expanded[axis + 2] - expanded[axis]) as isize
                - (plain[axis + 2] - plain[axis]) as isize;
            assert!(
                (1..=3).contains(&growth),
                "size {size}, axis {axis}: growth {growth}"
            );
        }
    }
    Ok(())
}

#[test]
fn hinted_and_unhinted_expansion_reserve_comparable_visible_space() -> Result<(), String> {
    for hint in [false, true] {
        let plain = bounds(32.0, 0.0, 1.0, hint)?;
        let expanded = bounds(32.0, 2.0, 1.0, hint)?;
        for axis in 0..2 {
            let growth = (expanded[axis + 2] - expanded[axis]) as isize
                - (plain[axis + 2] - plain[axis]) as isize;
            assert!(
                (3..=5).contains(&growth),
                "hint {hint}, axis {axis}: growth {growth}"
            );
        }
    }
    Ok(())
}
