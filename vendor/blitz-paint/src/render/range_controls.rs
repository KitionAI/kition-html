//! Native progress/meter tracks. Progress geometry follows Chromium's
//! NativeThemeBase::PaintProgressBar; its BSD terms are in LICENSE.chromium.
use super::ElementCx;
use crate::color::{Color, ToColorColor};
use anyrender::PaintScene;
use blitz_dom::CopyStyleGetters;
use kurbo::{Rect, Stroke};
use peniko::Fill;
use style::values::{computed::Appearance, generics::color::ColorOrAuto};

impl ElementCx<'_, '_> {
    pub(super) fn draw_range_control(&self, scene: &mut impl PaintScene) {
        let Some(range) = self.element.range_control() else {
            return;
        };
        let rect = self.frame.content_box;
        if rect.width() <= 0.0 || rect.height() <= 0.0 {
            return;
        }
        let vertical = !self.style.writing_mode.is_horizontal();
        let rtl = !self.style.writing_mode.is_bidi_ltr();
        let appearance_none = self.style.get_box().appearance == Appearance::None;
        let track = if appearance_none && range.progress {
            rect
        } else if vertical {
            Rect::new(
                rect.x0 + rect.width() * 0.25,
                rect.y0,
                rect.x1 - rect.width() * 0.25,
                rect.y1,
            )
        } else {
            Rect::new(
                rect.x0,
                rect.y0 + rect.height() * 0.25,
                rect.x1,
                rect.y1 - rect.height() * 0.25,
            )
        };
        if appearance_none && range.progress {
            scene.fill(
                Fill::NonZero,
                self.transform,
                Color::from_rgb8(128, 128, 128),
                None,
                &track,
            );
            if range.determinate {
                let value = range_rect(track, range.fraction(), vertical, rtl);
                scene.fill(
                    Fill::NonZero,
                    self.transform,
                    Color::from_rgb8(0, 128, 0),
                    None,
                    &value,
                );
            }
            return;
        }
        let radius = if range.progress { 40.0 } else { 20.0 } * self.scale;
        let rounded = track.to_rounded_rect(radius);
        scene.fill(
            Fill::NonZero,
            self.transform,
            Color::from_rgb8(239, 239, 239),
            None,
            &rounded,
        );
        if range.progress {
            let accent = match &self.style.get_inherited_ui().accent_color {
                ColorOrAuto::Auto => Color::from_rgb8(0, 117, 255),
                ColorOrAuto::Color(color) => color
                    .resolve_to_absolute(&self.style.clone_color())
                    .as_srgb_color(),
            };
            let mut fraction = if range.determinate {
                range.fraction()
            } else {
                0.2
            };
            let length = if vertical {
                track.height()
            } else {
                track.width()
            };
            let pixels = (length / self.scale * fraction).floor();
            fraction = if pixels > 0.0 {
                pixels.max(2.0) * self.scale / length
            } else {
                0.0
            };
            scene.push_clip_layer(Fill::NonZero, self.transform, &rounded);
            let value = range_rect(track, fraction, vertical, rtl);
            if range.determinate {
                scene.fill(Fill::NonZero, self.transform, accent, None, &value);
            } else {
                scene.fill(
                    Fill::NonZero,
                    self.transform,
                    accent,
                    None,
                    &value.to_rounded_rect(radius),
                );
            }
            scene.stroke(
                &Stroke::new(self.scale),
                self.transform,
                Color::from_rgba8(118, 118, 118, 128),
                None,
                &track
                    .inflate(-self.scale / 2., -self.scale / 2.)
                    .to_rounded_rect(radius),
            );
            scene.pop_layer();
        } else {
            scene.stroke(
                &Stroke::new(self.scale),
                self.transform,
                Color::from_rgba8(118, 118, 118, 77),
                None,
                &track
                    .inflate(-self.scale / 2., -self.scale / 2.)
                    .to_rounded_rect(radius),
            );
            let inner = track.inflate(-self.scale, -self.scale);
            if inner.width() <= 0. || inner.height() <= 0. {
                return;
            }
            let fill = match range.quality() {
                0 => Color::from_rgb8(16, 124, 16),
                1 => Color::from_rgb8(255, 185, 0),
                _ => Color::from_rgb8(216, 59, 1),
            };
            let length = if vertical {
                inner.height()
            } else {
                inner.width()
            };
            let fraction =
                (length / self.scale * range.fraction() * 64.).floor() / 64. * self.scale / length;
            let value = range_rect(inner, fraction, vertical, rtl);
            scene.push_clip_layer(
                Fill::NonZero,
                self.transform,
                &inner.to_rounded_rect((radius - self.scale).max(0.)),
            );
            scene.fill(Fill::NonZero, self.transform, fill, None, &value);
            scene.pop_layer();
        }
    }
}
fn range_rect(rect: Rect, fraction: f64, vertical: bool, reverse: bool) -> Rect {
    let fraction = fraction.clamp(0.0, 1.0);
    if vertical {
        if reverse {
            Rect::new(
                rect.x0,
                rect.y1 - rect.height() * fraction,
                rect.x1,
                rect.y1,
            )
        } else {
            Rect::new(
                rect.x0,
                rect.y0,
                rect.x1,
                rect.y0 + rect.height() * fraction,
            )
        }
    } else if reverse {
        Rect::new(rect.x1 - rect.width() * fraction, rect.y0, rect.x1, rect.y1)
    } else {
        Rect::new(rect.x0, rect.y0, rect.x0 + rect.width() * fraction, rect.y1)
    }
}
