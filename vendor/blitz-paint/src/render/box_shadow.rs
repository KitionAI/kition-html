use super::ElementCx;
use crate::color::{Color, ToColorColor as _};
use anyrender::PaintScene;
use blitz_dom::CopyStyleGetters;
use kurbo::{Affine, Rect, Shape as _, Vec2};
use peniko::{Fill, Mix};

impl ElementCx<'_, '_> {
    pub(super) fn draw_outset_box_shadow(&self, scene: &mut impl PaintScene) {
        let box_shadow = &self.style.get_effects().box_shadow.0;
        let has_outset_shadow = box_shadow.iter().any(|s| !s.inset);
        if !has_outset_shadow {
            return;
        }

        let current_color = self.style.clone_color();
        let opacity = self.style.get_effects().opacity;
        let bg_color = self
            .style
            .get_background()
            .background_color
            .resolve_to_absolute(&current_color)
            .as_srgb_color();
        let bg_is_opaque = bg_color.components[3] >= 1.0;
        let needs_clip = opacity < 1.0 || !bg_is_opaque;

        let max_shadow_rect = box_shadow.iter().fold(Rect::ZERO, |prev, shadow| {
            let x = shadow.base.horizontal.px() as f64 * self.scale;
            let y = shadow.base.vertical.px() as f64 * self.scale;
            let blur = shadow.base.blur.px() as f64 * self.scale;
            let spread = shadow.spread.px() as f64 * self.scale;
            let offset = spread + blur * 2.5;

            let rect = self.frame.border_box.inflate(offset, offset) + Vec2::new(x, y);

            prev.union(rect)
        });

        self.context.layer_manager.maybe_with_layer(
            scene,
            needs_clip,
            1.0,
            self.transform,
            &self.frame.shadow_clip(max_shadow_rect),
            None,
            None,
            |scene| {
                for shadow in box_shadow.iter().filter(|s| !s.inset).rev() {
                    let shadow_color = shadow
                        .base
                        .color
                        .resolve_to_absolute(&current_color)
                        .as_srgb_color();

                    let alpha = shadow_color.components[3];
                    if alpha != 0.0 {
                        let transform = self.transform
                            * Affine::translate(Vec2 {
                                x: shadow.base.horizontal.px() as f64 * self.scale,
                                y: shadow.base.vertical.px() as f64 * self.scale,
                            });

                        let spread = shadow.spread.px() as f64 * self.scale;
                        let blur = shadow.base.blur.px() as f64 * self.scale * 0.5;

                        // Without blur or spread the shadow is exactly the element's shape,
                        // so fill the border box path directly (preserving individual radii)
                        if blur == 0.0 && spread == 0.0 {
                            scene.fill(
                                Fill::NonZero,
                                transform,
                                shadow_color,
                                None,
                                &self.frame.border_box_path(),
                            );
                            continue;
                        }

                        let shape = self.frame.shadow_shape(false, spread);
                        draw_shadow_shape(scene, transform, &shape, shadow_color, blur);
                    }
                }
            },
        )
    }

    pub(super) fn draw_inset_box_shadow(&self, scene: &mut impl PaintScene) {
        let current_color = self.style.clone_color();
        let box_shadow = &self.style.get_effects().box_shadow.0;
        let has_inset_shadow = box_shadow.iter().any(|s| s.inset);
        if !has_inset_shadow {
            return;
        }

        let padding_box = self.frame.padding_box_path();

        for shadow in box_shadow.iter().filter(|s| s.inset).rev() {
            let shadow_color = shadow
                .base
                .color
                .resolve_to_absolute(&current_color)
                .as_srgb_color();
            if shadow_color.components[3] == 0.0 {
                continue;
            }

            let transform = self.transform
                * Affine::translate(Vec2 {
                    x: shadow.base.horizontal.px() as f64 * self.scale,
                    y: shadow.base.vertical.px() as f64 * self.scale,
                });

            scene.push_layer(
                Fill::NonZero,
                Mix::Normal,
                shadow_color.components[3],
                self.transform,
                &padding_box,
                None,
                None,
            );
            let spread = shadow.spread.px() as f64 * self.scale;
            let sigma = shadow.base.blur.px() as f64 * self.scale * 0.5;
            let hole = self.frame.shadow_shape(true, spread);
            // The exterior of the shifted padding box casts the inner shadow.
            // Rasterize its opaque mask first; shadow alpha is applied afterwards.
            let offset = shadow
                .base
                .horizontal
                .px()
                .abs()
                .max(shadow.base.vertical.px().abs()) as f64
                * self.scale;
            let outer = self.frame.padding_box.inflate(
                sigma * 4.0 + offset + spread.abs() + 1.0,
                sigma * 4.0 + offset + spread.abs() + 1.0,
            );
            let mut path = outer.to_path(0.1);
            if hole.border_box.width() > 0.0 && hole.border_box.height() > 0.0 {
                path.extend(hole.border_box_path().elements().iter().copied());
                path.close_path();
            }
            let filter = (sigma > 0.0).then(|| {
                std::sync::Arc::new(anyrender::filters::Filter::linear_list(std::iter::once(
                    anyrender::filters::FilterEffect::blur(sigma as f32),
                )))
            });
            scene.push_layer(
                Fill::NonZero,
                Mix::Normal,
                1.0,
                transform,
                &outer,
                filter,
                None,
            );
            let mut opaque = shadow_color;
            opaque.components[3] = 1.0;
            scene.fill(Fill::EvenOdd, transform, opaque, None, &path);
            scene.pop_layer();
            scene.pop_layer();
        }
    }
}

/// Preserve CSS corner geometry before applying the shadow's Gaussian blur.
fn draw_shadow_shape(
    scene: &mut impl PaintScene,
    transform: Affine,
    shape: &crate::kurbo_css::CssBox,
    color: Color,
    sigma: f64,
) {
    let rect = shape.border_box;
    if rect.width() <= 0.0 || rect.height() <= 0.0 {
        return;
    }
    if sigma == 0.0 {
        scene.fill(
            Fill::NonZero,
            transform,
            color,
            None,
            &shape.border_box_path(),
        );
        return;
    }
    let filter = anyrender::filters::Filter::linear_list(
        [anyrender::filters::FilterEffect::blur(sigma as f32)].into_iter(),
    );
    scene.push_layer(
        Fill::NonZero,
        Mix::Normal,
        1.0,
        transform,
        &rect.inflate(sigma * 4.0, sigma * 4.0),
        Some(std::sync::Arc::new(filter)),
        None,
    );
    scene.fill(
        Fill::NonZero,
        transform,
        color,
        None,
        &shape.border_box_path(),
    );
    scene.pop_layer();
}
