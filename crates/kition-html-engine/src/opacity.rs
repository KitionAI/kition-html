//! Fold a single solid draw's opacity without changing its clip stack.
use anyrender::{Paint, Scene, recording::RenderCommand};
use peniko::BlendMode;

#[derive(Default)]
struct Group {
    start: usize,
    draw: Option<usize>,
    multiple: bool,
    blocked: bool,
}

pub fn prepare(scene: &mut Scene) {
    let mut stack: Vec<Group> = Vec::new();
    for index in 0..scene.commands.len() {
        match &scene.commands[index] {
            RenderCommand::PushLayer(layer) => {
                stack.push(Group {
                    start: index,
                    blocked: layer.blend != BlendMode::default()
                        || layer.filter.is_some()
                        || layer.backdrop_filter.is_some(),
                    ..Default::default()
                });
            }
            RenderCommand::PushClipLayer(_) => stack.push(Group {
                start: index,
                ..Default::default()
            }),
            RenderCommand::PopLayer => {
                let Some(group) = stack.pop() else {
                    continue;
                };
                let nested_opacity = matches!(&scene.commands[group.start],
                    RenderCommand::PushLayer(layer) if layer.alpha < 1.0);
                if !group.multiple
                    && !group.blocked
                    && let Some(draw) = group.draw
                {
                    let alpha = match &scene.commands[group.start] {
                        RenderCommand::PushLayer(layer) => layer.alpha,
                        _ => 1.0,
                    };
                    if alpha < 1.0
                        && let RenderCommand::Fill(fill) = &mut scene.commands[draw]
                        && let Paint::Solid(color) = &mut fill.brush
                    {
                        *color = color.multiply_alpha(alpha);
                        if let RenderCommand::PushLayer(layer) = &mut scene.commands[group.start] {
                            layer.alpha = 1.0;
                        }
                    }
                }
                if let Some(parent) = stack.last_mut() {
                    parent.blocked |= group.blocked || nested_opacity;
                    parent.multiple |=
                        group.multiple || (parent.draw.is_some() && group.draw.is_some());
                    if group.draw.is_some() {
                        parent.draw = group.draw;
                    }
                }
            }
            command => {
                if let Some(group) = stack.last_mut() {
                    group.multiple |= group.draw.is_some();
                    group.draw = Some(index);
                    group.blocked |= !matches!(command, RenderCommand::Fill(fill) if matches!(fill.brush, Paint::Solid(_)));
                }
            }
        }
    }
    // Quantize alpha before premultiplying RGB, as an RGBA8 paint does. Using
    // the original fractional alpha for RGB can lose a channel level after the
    // rounded alpha is composited over the backdrop.
    for command in &mut scene.commands {
        let brush = match command {
            RenderCommand::Fill(fill) => Some(&mut fill.brush),
            RenderCommand::Stroke(stroke) => Some(&mut stroke.brush),
            RenderCommand::GlyphRun(glyphs) => Some(&mut glyphs.brush),
            _ => None,
        };
        if let Some(Paint::Solid(color)) = brush {
            color.components[3] = (color.components[3] * 255.0).round() / 255.0;
        }
    }
}
