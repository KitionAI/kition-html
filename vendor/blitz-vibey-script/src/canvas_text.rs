//! Canvas text uses the document's supplied fonts and the shared CPU glyph path.
use crate::state::DomCtx;
use anyrender::{Paint, PaintScene};
use blitz_dom::util::{glyph_embolden, snap_glyph_baseline};
use boa_engine::{JsNativeError, JsResult};
use parley::{
    Alignment, AlignmentOptions, BaseDirection, FontFamily, FontFeatures, FontStyle,
    FontVariations, FontWeight, Layout, LayoutContext, PositionedLayoutItem, StyleProperty,
};
use peniko::{
    Fill, StyleRef,
    kurbo::{Affine, Rect, Stroke, Vec2},
};
use serde_json::{Value, json};
use skrifa::{
    FontRef, GlyphId, MetadataProvider,
    instance::{LocationRef, Size},
};
use std::borrow::Cow;

pub(crate) struct TextLayout {
    layout: Layout<()>,
    width: f64,
    baseline: f64,
    ascent: f64,
    descent: f64,
    bounds: Option<Rect>,
    size: f64,
}

pub(crate) fn shape(ctx: &DomCtx, request: &Value) -> JsResult<TextLayout> {
    let input = request["text"].as_str().unwrap_or("");
    if input.len() > 262144 {
        return Err(JsNativeError::range()
            .with_message("Canvas text input limit exceeded")
            .into());
    }
    let text: String = input
        .chars()
        .map(|ch| {
            if matches!(ch, '\t' | '\n' | '\r' | '\u{000c}') {
                ' '
            } else {
                ch
            }
        })
        .collect();
    let state = &request["state"];
    let font = &state["fontData"];
    let size = font["size"].as_f64().unwrap_or(10.);
    if !size.is_finite() || size <= 0. || size > 8192. {
        return Err(JsNativeError::range()
            .with_message("Canvas font size exceeds the preview limit")
            .into());
    }
    let (letter_spacing, word_spacing) = crate::canvas_spacing::resolve(ctx, state, size)?;
    let fonts = ctx.doc.borrow().font_context();
    let mut fonts = fonts
        .lock()
        .map_err(|_| JsNativeError::error().with_message("Font resources are unavailable"))?;
    let mut context = LayoutContext::new();
    let mut builder = context.ranged_builder(&mut fonts, &text, 1., false);
    builder.set_base_direction(if state["direction"] == "rtl" {
        BaseDirection::Rtl
    } else {
        BaseDirection::Ltr
    });
    builder.push_default(StyleProperty::FontFamily(FontFamily::Source(
        Cow::Borrowed(font["family"].as_str().unwrap_or("sans-serif")),
    )));
    builder.push_default(StyleProperty::FontSize(size as f32));
    builder.push_default(StyleProperty::FontWeight(FontWeight::new(
        font["weight"].as_f64().unwrap_or(400.) as f32,
    )));
    builder.push_default(StyleProperty::FontStyle(
        FontStyle::parse_css(font["style"].as_str().unwrap_or("normal")).unwrap_or_default(),
    ));
    let variations = format!("'opsz' {size}");
    builder.push_default(StyleProperty::FontVariations(FontVariations::from(
        variations.as_str(),
    )));
    // Kerning is a shaping feature, shared by measurement and both paint paths.
    // Preserve optional ligatures and every other default font feature.
    let kerning = match state["fontKerning"].as_str() {
        Some("none") => Some("'kern' 0"),
        Some("normal") => Some("'kern' 1"),
        _ => None,
    };
    if let Some(kerning) = kerning {
        builder.push_default(StyleProperty::FontFeatures(FontFeatures::from(kerning)));
    }
    builder.push_default(StyleProperty::LetterSpacing(letter_spacing));
    builder.push_default(StyleProperty::WordSpacing(word_spacing));
    let mut layout: Layout<()> = builder.build(&text);
    layout.break_all_lines(None);
    layout.align(Alignment::Start, AlignmentOptions::default());
    let width = if text.is_empty() {
        0.
    } else {
        let full = layout.full_width();
        // Canvas can report negative advances when spacing reverses the run.
        if full == 0.0 && (letter_spacing < 0.0 || word_spacing < 0.0) {
            layout
                .lines()
                .next()
                .map_or(0.0, |line| f64::from(line.metrics().advance))
        } else {
            f64::from(full)
        }
    };
    let baseline = layout
        .lines()
        .next()
        .map_or(0., |line| line.metrics().baseline as f64);
    let mut ascent = 0_f64;
    let mut descent = 0_f64;
    let mut bounds: Option<Rect> = None;
    for line in layout.lines() {
        for item in line.items() {
            let PositionedLayoutItem::GlyphRun(glyphs) = item else {
                continue;
            };
            let run = glyphs.run();
            ascent = ascent.max(run.font_metrics().ascent as f64);
            descent = descent.max(run.font_metrics().descent as f64);
            if let Ok(font) = FontRef::from_index(run.font().data.data(), run.font().index) {
                let coords: Vec<_> = run
                    .normalized_coords()
                    .iter()
                    .map(|c| skrifa::instance::NormalizedCoord::from_bits(c.to_bits()))
                    .collect();
                let metrics =
                    font.glyph_metrics(Size::new(run.font_size()), LocationRef::new(&coords));
                for glyph in glyphs.positioned_glyphs() {
                    if let Some(boxes) = metrics.bounds(GlyphId::new(glyph.id)) {
                        let rect = Rect::new(
                            (glyph.x + boxes.x_min) as f64,
                            glyph.y as f64 - baseline - boxes.y_max as f64,
                            (glyph.x + boxes.x_max) as f64,
                            glyph.y as f64 - baseline - boxes.y_min as f64,
                        );
                        bounds = Some(bounds.map_or(rect, |old| old.union(rect)));
                    }
                }
            }
        }
    }
    Ok(TextLayout {
        layout,
        width,
        baseline,
        ascent,
        descent,
        bounds,
        size,
    })
}

impl TextLayout {
    pub(crate) fn spacing_font_metrics(&self) -> Option<(f64, f64, f64)> {
        for line in self.layout.lines() {
            for item in line.items() {
                if let PositionedLayoutItem::GlyphRun(glyphs) = item {
                    let metrics = glyphs.run().font_metrics();
                    return Some((
                        f64::from(metrics.x_height.unwrap_or(self.size as f32 * 0.5)),
                        f64::from(metrics.cap_height.unwrap_or(metrics.ascent)),
                        self.width,
                    ));
                }
            }
        }
        None
    }

    fn anchor(&self, state: &Value) -> (f64, f64) {
        let rtl = state["direction"] == "rtl";
        let x = match state["textAlign"].as_str().unwrap_or("start") {
            "center" => -self.width / 2.,
            "right" => -self.width,
            "start" if rtl => -self.width,
            "end" if !rtl => -self.width,
            _ => 0.,
        };
        let em_ascent = if self.ascent + self.descent > 0. {
            (self.size * self.ascent / (self.ascent + self.descent) * 64.).round() / 64.
        } else {
            0.
        };
        let y = match state["textBaseline"].as_str().unwrap_or("alphabetic") {
            "top" => em_ascent,
            "hanging" => self.ascent.round() * 0.8,
            "middle" => em_ascent - self.size / 2.,
            "bottom" => em_ascent - self.size,
            "ideographic" => -self.descent.round(),
            _ => 0.,
        };
        (x, y)
    }
    pub(crate) fn metrics(&self, state: &Value) -> Value {
        let (x, y) = self.anchor(state);
        let bounds = self.bounds.unwrap_or(Rect::ZERO);
        json!({"width":self.width,"actualBoundingBoxLeft":-bounds.x0-x,"actualBoundingBoxRight":bounds.x1+x,
            "actualBoundingBoxAscent":-bounds.y0-y,"actualBoundingBoxDescent":bounds.y1+y,
            "fontBoundingBoxAscent":self.ascent.round()-y,"fontBoundingBoxDescent":self.descent.round()+y,
            "hangingBaseline":self.ascent.round()*0.8-y,"alphabeticBaseline":-y,"ideographicBaseline":-self.descent.round()-y})
    }
    pub(crate) fn is_empty(&self) -> bool {
        self.bounds.is_none()
    }
    pub(crate) fn draw(
        &self,
        scene: &mut impl PaintScene,
        request: &Value,
        transform: Affine,
        paint: &Paint,
        stroke: Option<&Stroke>,
    ) {
        let state = &request["state"];
        let args = &request["args"];
        let x = args[0].as_f64().unwrap_or(0.);
        let y = args[1].as_f64().unwrap_or(0.);
        let max = args[2].as_f64().unwrap_or(self.width);
        let sx = if self.width > 0. {
            (max / self.width).min(1.)
        } else {
            1.
        };
        let (ax, ay) = self.anchor(state);
        let transform = transform
            * Affine::translate((x, y))
            * Affine::scale_non_uniform(sx, 1.)
            * Affine::translate((ax, ay - self.baseline));
        let style = stroke.map_or(StyleRef::Fill(Fill::NonZero), StyleRef::Stroke);
        // Horizontal fill and stroke masks quantize in bitmap space after
        // alignment, maxWidth, and the current transform.
        let snap_origins = transform.as_coeffs()[1].abs() < 1e-8;
        for line in self.layout.lines() {
            for item in line.items() {
                let PositionedLayoutItem::GlyphRun(glyphs) = item else {
                    continue;
                };
                let run = glyphs.run();
                let coords: Vec<i16> = run
                    .normalized_coords()
                    .iter()
                    .map(|c| c.to_bits())
                    .collect();
                let skew = run
                    .synthesis()
                    .skew()
                    .map(|angle| Affine::skew(-angle.to_radians().tan() as f64, 0.));
                let embolden = if stroke.is_some() {
                    Vec2::ZERO
                } else {
                    glyph_embolden(run.font_size(), transform)
                };
                scene.draw_glyphs(
                    run.font(),
                    run.font_size(),
                    false,
                    &coords,
                    embolden,
                    style,
                    paint.as_ref(),
                    1.,
                    transform,
                    skew,
                    glyphs.positioned_glyphs().map(|glyph| {
                        let (x, y) = if snap_origins {
                            snap_glyph_baseline(transform, glyph.x, glyph.y)
                        } else {
                            (glyph.x, glyph.y)
                        };
                        anyrender::Glyph { id: glyph.id, x, y }
                    }),
                );
            }
        }
    }
}
