//! Nine-slice border images. Source coordinates remain independent of border widths.
use anyrender::{Paint, PaintScene};
use blitz_dom::{CopyStyleGetters, node::ImageData};
use kurbo::{Affine, Insets, Rect};
use peniko::Fill;
use style::values::{
    computed::{BorderImageSideWidth, Length, NonNegativeLengthOrNumber, NumberOrPercentage},
    generics::{
        border::GenericBorderImageSideWidth, image::GenericImage, length::GenericLengthOrNumber,
    },
    specified::border::BorderImageRepeatKeyword as Repeat,
};

use super::{ElementCx, to_image_quality, to_peniko_image};
use crate::gradient::to_peniko_gradient;

impl ElementCx<'_, '_> {
    /// Return true when an available image replaces the ordinary border.
    pub(super) fn draw_border_image(&self, scene: &mut impl PaintScene) -> bool {
        let border = self.style.get_border();
        if matches!(&border.border_image_source, GenericImage::None) {
            return false;
        }
        let bw = self.frame.border_width;
        let outset = &border.border_image_outset;
        let resolve_outset = |value: &NonNegativeLengthOrNumber, width| match value {
            GenericLengthOrNumber::Number(value) => f64::from(value.0) * width,
            GenericLengthOrNumber::Length(value) => f64::from(value.0.px()) * self.scale,
        };
        let area = self.frame.border_box
            + Insets::new(
                resolve_outset(&outset.3, bw.x0),
                resolve_outset(&outset.0, bw.y0),
                resolve_outset(&outset.1, bw.x1),
                resolve_outset(&outset.2, bw.y1),
            );
        if area.width() <= 0.0 || area.height() <= 0.0 {
            return true;
        }
        let (paint, source, number_scale, intrinsic) = match &border.border_image_source {
            GenericImage::Gradient(gradient) => {
                let source = Rect::new(0.0, 0.0, area.width(), area.height());
                let (gradient, transform) = to_peniko_gradient(
                    gradient,
                    source,
                    source,
                    self.scale,
                    &self.style.clone_color(),
                );
                (
                    BorderSource::Gradient(gradient, transform.unwrap_or(Affine::IDENTITY)),
                    source,
                    self.scale,
                    false,
                )
            }
            GenericImage::Url(_) => {
                let Some(Some(resource)) = self.element.border_images.first() else {
                    return false;
                };
                match &resource.image {
                    ImageData::Raster(image) => {
                        let source =
                            Rect::new(0.0, 0.0, f64::from(image.width), f64::from(image.height));
                        (BorderSource::Raster(image), source, 1.0, true)
                    }
                    #[cfg(feature = "svg")]
                    ImageData::Svg(svg) => {
                        if svg.intrinsic_dimensions.degenerate_view_box {
                            return true;
                        }
                        let (width, height) = super::background::default_sizing(
                            svg.intrinsic_width(),
                            svg.intrinsic_height(),
                            svg.viewbox_aspect_ratio(),
                            (area.width() / self.scale) as f32,
                            (area.height() / self.scale) as f32,
                        );
                        let source = Rect::new(0.0, 0.0, f64::from(width), f64::from(height));
                        (
                            BorderSource::Svg(svg, super::svg_viewport_affine(svg, width, height)),
                            source,
                            1.0,
                            true,
                        )
                    }
                    _ => return false,
                }
            }
            _ => {
                self.context.diagnostics.borrow_mut().push("html.unsupported-border-image: this border image source type is not implemented".into());
                return false;
            }
        };
        if source.width() <= 0.0 || source.height() <= 0.0 {
            return true;
        }
        let slice = &border.border_image_slice.offsets;
        let resolve_slice = |value: &NumberOrPercentage, dimension: f64| match value {
            NumberOrPercentage::Number(value) => (f64::from(*value) * number_scale).min(dimension),
            NumberOrPercentage::Percentage(value) => {
                (f64::from(value.0) * dimension).min(dimension)
            }
        };
        let slices = [
            resolve_slice(&slice.0.0, source.height()),
            resolve_slice(&slice.1.0, source.width()),
            resolve_slice(&slice.2.0, source.height()),
            resolve_slice(&slice.3.0, source.width()),
        ];
        let widths = &border.border_image_width;
        let resolve_width =
            |value: &BorderImageSideWidth, dimension: f64, fallback, slice: f64| match value {
                GenericBorderImageSideWidth::Number(value) => f64::from(value.0) * fallback,
                GenericBorderImageSideWidth::LengthPercentage(value) => {
                    f64::from(
                        value
                            .0
                            .resolve(Length::new((dimension / self.scale) as f32))
                            .px(),
                    ) * self.scale
                }
                GenericBorderImageSideWidth::Auto => {
                    if intrinsic {
                        slice * self.scale
                    } else {
                        fallback
                    }
                }
            };
        let mut widths = [
            resolve_width(&widths.0, area.height(), bw.y0, slices[0]),
            resolve_width(&widths.1, area.width(), bw.x1, slices[1]),
            resolve_width(&widths.2, area.height(), bw.y1, slices[2]),
            resolve_width(&widths.3, area.width(), bw.x0, slices[3]),
        ];
        let factor = (area.width() / (widths[1] + widths[3]))
            .min(area.height() / (widths[0] + widths[2]))
            .min(1.0);
        for width in &mut widths {
            *width *= factor;
        }
        let sx = [0.0, slices[3], source.width() - slices[1], source.width()];
        let sy = [0.0, slices[0], source.height() - slices[2], source.height()];
        let dx = [area.x0, area.x0 + widths[3], area.x1 - widths[1], area.x1];
        let dy = [area.y0, area.y0 + widths[0], area.y1 - widths[2], area.y1];
        let repeat = &border.border_image_repeat;
        let horizontal_scale = if slices[0] > 0.0 && widths[0] > 0.0 {
            widths[0] / slices[0]
        } else if slices[2] > 0.0 && widths[2] > 0.0 {
            widths[2] / slices[2]
        } else {
            1.0
        };
        let vertical_scale = if slices[3] > 0.0 && widths[3] > 0.0 {
            widths[3] / slices[3]
        } else if slices[1] > 0.0 && widths[1] > 0.0 {
            widths[1] / slices[1]
        } else {
            1.0
        };
        for row in 0..3 {
            for col in 0..3 {
                if row == 1 && col == 1 && !border.border_image_slice.fill {
                    continue;
                }
                let src = Rect::new(sx[col], sy[row], sx[col + 1], sy[row + 1]);
                let dst = Rect::new(dx[col], dy[row], dx[col + 1], dy[row + 1]);
                if src.width() <= 0.0
                    || src.height() <= 0.0
                    || dst.width() <= 0.0
                    || dst.height() <= 0.0
                {
                    continue;
                }
                let natural_width = src.width()
                    * if row != 1 {
                        dst.height() / src.height()
                    } else {
                        horizontal_scale
                    };
                let natural_height = src.height()
                    * if col != 1 {
                        dst.width() / src.width()
                    } else {
                        vertical_scale
                    };
                let x = tiles(
                    dst.x0,
                    dst.width(),
                    natural_width,
                    if col == 1 { repeat.0 } else { Repeat::Stretch },
                );
                let y = tiles(
                    dst.y0,
                    dst.height(),
                    natural_height,
                    if row == 1 { repeat.1 } else { Repeat::Stretch },
                );
                if u64::from(x.count) * u64::from(y.count) > 500 {
                    self.context.diagnostics.borrow_mut().push("html.border-image-tile-limit: border image patch exceeds 500 tiles; patch omitted".into());
                    continue;
                }
                scene.push_clip_layer(Fill::NonZero, self.transform, &dst);
                for iy in 0..y.count {
                    for ix in 0..x.count {
                        let tile = Rect::new(
                            x.start + f64::from(ix) * x.stride,
                            y.start + f64::from(iy) * y.stride,
                            x.start + f64::from(ix) * x.stride + x.size,
                            y.start + f64::from(iy) * y.stride + y.size,
                        );
                        let mapping = Affine::translate(tile.origin().to_vec2())
                            * Affine::scale_non_uniform(
                                tile.width() / src.width(),
                                tile.height() / src.height(),
                            )
                            * Affine::translate(-src.origin().to_vec2());
                        match &paint {
                            BorderSource::Gradient(gradient, transform) => scene.fill(
                                Fill::NonZero,
                                self.transform,
                                Paint::Gradient(gradient),
                                Some(mapping * *transform),
                                &tile,
                            ),
                            BorderSource::Raster(image) => scene.fill(
                                Fill::NonZero,
                                self.transform,
                                Paint::Image(
                                    to_peniko_image(
                                        image,
                                        to_image_quality(
                                            self.style.get_inherited_box().image_rendering,
                                        ),
                                    )
                                    .as_ref(),
                                ),
                                Some(mapping),
                                &tile,
                            ),
                            #[cfg(feature = "svg")]
                            BorderSource::Svg(svg, viewport) => {
                                scene.push_clip_layer(Fill::NonZero, self.transform, &tile);
                                anyrender_svg::render_svg_tree(
                                    scene,
                                    &svg.tree,
                                    self.transform * mapping * *viewport,
                                );
                                scene.pop_layer();
                            }
                        }
                    }
                }
                scene.pop_layer();
            }
        }
        true
    }
}

enum BorderSource<'a> {
    Gradient(peniko::Gradient, Affine),
    Raster(&'a blitz_dom::node::RasterImageData),
    #[cfg(feature = "svg")]
    Svg(&'a blitz_dom::node::SvgImageData, Affine),
}

struct Tiles {
    start: f64,
    size: f64,
    stride: f64,
    count: u32,
}

fn tiles(start: f64, length: f64, natural: f64, mode: Repeat) -> Tiles {
    if !natural.is_finite() || natural <= 0.0 || mode == Repeat::Stretch {
        return Tiles {
            start,
            size: length,
            stride: length,
            count: 1,
        };
    }
    match mode {
        Repeat::Round => {
            let count = (length / natural).round().max(1.0).min(501.0) as u32;
            let size = length / f64::from(count);
            Tiles {
                start,
                size,
                stride: size,
                count,
            }
        }
        Repeat::Space => {
            let count = (length / natural).floor().min(501.0) as u32;
            let gap = (length - f64::from(count) * natural) / (f64::from(count) + 1.0);
            Tiles {
                start: start + gap,
                size: natural,
                stride: natural + gap,
                count,
            }
        }
        Repeat::Repeat => {
            let extra = ((length - natural) / (2.0 * natural))
                .ceil()
                .clamp(0.0, 250.0) as u32;
            let count = extra * 2 + 1;
            Tiles {
                start: start + (length - natural) / 2.0 - f64::from(extra) * natural,
                size: natural,
                stride: natural,
                count,
            }
        }
        Repeat::Stretch => unreachable!(),
    }
}
