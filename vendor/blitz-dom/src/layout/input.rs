//! Intrinsic height and inline baseline for single-line text controls.
use super::resolve_calc_value;
use crate::node::Node;
use style::values::computed::Appearance;
use taffy::{
    AvailableSpace, Baselines, CoreStyle, LayoutInput, LayoutOutput, ResolveOrZero, Size,
    compute_leaf_layout,
};

/// Native checkable controls align their border-box bottom to the text
/// baseline. An unthemed control is an ordinary empty inline block instead.
pub(super) fn checkable(node: &Node, inputs: LayoutInput) -> LayoutOutput {
    let native = node
        .primary_styles()
        .is_some_and(|style| style.get_box().appearance != Appearance::None);
    let intrinsic = if native { 13. } else { 0. };
    let mut output =
        compute_leaf_layout(inputs, &node.layout_style(), resolve_calc_value, |_, _| {
            Size {
                width: intrinsic,
                height: intrinsic,
            }
        });
    if native {
        output.baselines = Baselines {
            first: Some(output.size.height),
            last: Some(output.size.height),
        };
    }
    output
}

pub(super) fn single_line(
    node: &Node,
    inputs: LayoutInput,
    scale: f32,
    fallback_height: f32,
) -> LayoutOutput {
    let layout = node
        .element_data()
        .and_then(|element| element.text_input_data())
        .and_then(|input| input.editor.try_layout());
    let line = layout.and_then(|layout| layout.lines().next());
    let font_height =
        line.as_ref()
            .and_then(|line| line.runs().next())
            .map_or(fallback_height, |run| {
                (run.font_metrics().ascent / scale).round()
                    + (run.font_metrics().descent / scale).round()
            });
    let text_height = layout.map_or(font_height, |layout| layout.height() / scale);
    let text_baseline = line.map_or(font_height, |line| line.metrics().baseline / scale);
    // CSS line-height may enlarge an input, but cannot shrink its intrinsic
    // content height below the font's normal ascent and descent.
    let height = font_height.max(text_height);
    let style = node.layout_style();
    let padding = style
        .padding()
        .resolve_or_zero(inputs.parent_size.width, resolve_calc_value);
    let border = style
        .border()
        .resolve_or_zero(inputs.parent_size.width, resolve_calc_value);
    let mut output = compute_leaf_layout(inputs, &style, resolve_calc_value, |_, _| Size {
        width: match inputs.available_space.width {
            AvailableSpace::Definite(limit) => limit.min(300.),
            AvailableSpace::MinContent => 0.,
            AvailableSpace::MaxContent => 300.,
        },
        height,
    });
    let content_height =
        output.size.height - padding.vertical_axis_sum() - border.vertical_axis_sum();
    let baseline =
        border.top + padding.top + ((content_height - text_height) / 2.).max(0.) + text_baseline;
    output.baselines = Baselines {
        first: Some(baseline),
        last: Some(baseline),
    };
    output
}
