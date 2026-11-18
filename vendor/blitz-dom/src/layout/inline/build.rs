use blitz_traits::node_id::NodeId;
use markup5ever::local_name;
use parley::{
    FontContext, InlineBox, InlineBoxKind, LayoutContext, TreeBuilder, WhiteSpaceCollapse,
};
use style::{
    computed_values::position::T as PositionProperty,
    values::{
        computed::{Display, Float},
        specified::box_::{DisplayInside, DisplayOutside},
    },
};
use stylo_taffy::CopyStyleGetters;

use crate::{
    NodeData,
    layout::list::BULLET_FONT_FAMILY,
    layout::replaced::is_inline_box_element,
    layout::text_transform::{CaseTransform, TextTransformer},
    node::{ListItemLayout, ListItemLayoutPosition, Marker, TextBrush, TextLayout},
    stylo_to_parley,
};

pub(crate) fn build_inline_layout_into(
    nodes: &crate::NodeTree,
    layout_ctx: &mut LayoutContext<TextBrush>,
    font_ctx: &mut FontContext,
    text_layout: &mut TextLayout,
    scale: f32,
    inline_context_root_node_id: NodeId,
) {
    // Get the inline context's root node's text styles
    let root_node = &nodes[inline_context_root_node_id];
    let root_node_style = root_node.primary_styles().or_else(|| {
        root_node
            .parent
            .and_then(|parent_id| nodes[parent_id].primary_styles())
    });

    let parley_style = root_node_style
        .as_ref()
        .map(|s| stylo_to_parley::style(inline_context_root_node_id, s))
        .unwrap_or_else(|| parley::TextStyle {
            white_space_collapse: WhiteSpaceCollapse::Collapse,
            ..Default::default()
        });

    let symbol_style = root_node
        .element_data()
        .and_then(|el| el.list_item_data.as_deref())
        .filter(|item| matches!(item.position, ListItemLayoutPosition::Inside))
        .and_then(|item| {
            crate::layout::list::symbol_marker_style(
                layout_ctx,
                font_ctx,
                &parley_style,
                &item.marker,
                true,
                scale,
            )
        });

    // Create a parley tree builder
    let mut builder = layout_ctx.tree_builder(font_ctx, scale, true, &parley_style);
    if let Some(style) = root_node_style.as_deref() {
        builder.set_base_direction(stylo_to_parley::base_direction(
            style.clone_direction(),
            style.clone_unicode_bidi(),
        ));
    }

    let text_transform = root_node_style
        .as_deref()
        .map(|s| CaseTransform::from_style(s))
        .unwrap_or(CaseTransform::NONE);

    let (bidi_start, bidi_end) = root_node_style
        .as_deref()
        .map(|style| bidi_controls(style, true))
        .unwrap_or_default();
    builder.push_text(bidi_start);

    // Render position-inside list items
    if let Some(ListItemLayout {
        marker,
        position: ListItemLayoutPosition::Inside,
    }) = root_node
        .element_data()
        .and_then(|el| el.list_item_data.as_deref())
    {
        match marker {
            Marker::Char(char) => {
                if let Some(marker_style) = symbol_style {
                    builder.push_style_span(marker_style);
                    builder.push_text("\u{a0}");
                    builder.pop_style_span();
                } else {
                    let mut marker_style = parley_style.clone();
                    marker_style.font_family = BULLET_FONT_FAMILY.into();
                    builder.push_style_span(marker_style);
                    builder.push_text(&format!("{char} "));
                    builder.pop_style_span();
                }
            }
            Marker::String(str) => {
                builder.push_style_span(crate::layout::list::textual_marker_style(&parley_style));
                builder.push_text(str);
                builder.pop_style_span();
            }
        }
    };
    // The marker is a separate box, so words in the content don't continue from it.
    let mut text_transformer = TextTransformer::default();
    text_transformer.word_break(&builder);

    if let Some(before_id) = root_node.before() {
        build_inline_layout_recursive(
            &mut builder,
            &mut text_transformer,
            nodes,
            before_id,
            &text_transform,
        );
    }
    for child_id in root_node.children.iter().copied() {
        build_inline_layout_recursive(
            &mut builder,
            &mut text_transformer,
            nodes,
            child_id,
            &text_transform,
        );
    }
    if let Some(after_id) = root_node.after() {
        build_inline_layout_recursive(
            &mut builder,
            &mut text_transformer,
            nodes,
            after_id,
            &text_transform,
        );
    }

    builder.push_text(bidi_end);
    text_layout.text = builder.build_into(&mut text_layout.layout);
    return;

    fn build_inline_layout_recursive(
        builder: &mut TreeBuilder<TextBrush>,
        text_transformer: &mut TextTransformer,
        nodes: &crate::NodeTree,
        node_id: NodeId,
        parent_text_transform: &CaseTransform,
    ) {
        let node = &nodes[node_id];

        let style = node.primary_styles();
        let style = style.as_ref();

        let text_transform = style
            .map(|s| CaseTransform::from_style(s))
            .unwrap_or(CaseTransform::NONE);

        match &node.data {
            NodeData::Element(element_data) | NodeData::AnonymousBlock(element_data) => {
                // if the input type is hidden, hide it
                if *element_data.name.local == *"input" {
                    if let Some("hidden") = element_data.attr(local_name!("type")) {
                        return;
                    }
                }

                let display = node.display_style().unwrap_or(Display::inline());
                let position = style
                    .map(|s| s.clone_position())
                    .unwrap_or(PositionProperty::Static);
                let float = style.map(|s| s.clone_float()).unwrap_or(Float::None);
                let box_kind = if position.is_absolutely_positioned() {
                    InlineBoxKind::OutOfFlow
                } else if float.is_floating() {
                    InlineBoxKind::CustomOutOfFlow
                } else {
                    InlineBoxKind::InFlow
                };

                // HTML wbr is a shaping control even when positioned or floated.
                // display:none/contents still remove its own rendering object.
                if element_data.name.local == local_name!("wbr")
                    && !matches!(display.outside(), DisplayOutside::None)
                {
                    builder.push_style_modification_span(&[
                        parley::StyleProperty::TextWrapMode(parley::TextWrapMode::Wrap),
                        parley::StyleProperty::LetterSpacing(0.0),
                    ]);
                    text_transformer.discretionary_break(builder);
                    builder.pop_style_span();
                    return;
                }

                match (display.outside(), display.inside()) {
                    (DisplayOutside::None, DisplayInside::None) => {
                        // node.remove_damage(CONSTRUCT_DESCENDENT | CONSTRUCT_FC | CONSTRUCT_BOX);
                    }
                    (DisplayOutside::None, DisplayInside::Contents) => {
                        let whitespace = style.map(|s| {
                            [
                                parley::StyleProperty::WhiteSpaceCollapse(
                                    stylo_to_parley::white_space_collapse(
                                        s.clone_white_space_collapse(),
                                    ),
                                ),
                                parley::StyleProperty::TextWrapMode(
                                    stylo_to_parley::text_wrap_mode(s.clone_text_wrap_mode()),
                                ),
                            ]
                        });
                        builder.push_style_modification_span(
                            whitespace
                                .as_ref()
                                .map_or(&[][..], |styles| styles.as_slice()),
                        );
                        for child_id in node.children.iter().copied() {
                            // node.remove_damage(CONSTRUCT_DESCENDENT | CONSTRUCT_FC | CONSTRUCT_BOX);
                            build_inline_layout_recursive(
                                builder,
                                text_transformer,
                                nodes,
                                child_id,
                                &text_transform,
                            );
                        }
                        builder.pop_style_span();
                    }
                    (DisplayOutside::Inline, DisplayInside::Flow) => {
                        let tag_name = &element_data.name.local;

                        if is_inline_box_element(tag_name) {
                            if box_kind == InlineBoxKind::InFlow {
                                text_transformer.word_break(builder);
                            }
                            builder.push_inline_box(InlineBox {
                                id: node_id.as_u64(),
                                kind: box_kind,
                                // Overridden by push_inline_box method
                                index: 0,
                                // Width and height are set during layout
                                width: 0.0,
                                height: 0.0,
                                baseline: None,
                                vertical_align: node
                                    .primary_styles()
                                    .map(|s| stylo_to_parley::vertical_align(&s))
                                    .unwrap_or_default(),
                            });
                        } else if *tag_name == local_name!("br") {
                            // node.remove_damage(CONSTRUCT_DESCENDENT | CONSTRUCT_FC | CONSTRUCT_BOX);
                            // TODO: update span id for br spans
                            builder.push_style_modification_span(&[
                                parley::StyleProperty::WhiteSpaceCollapse(
                                    WhiteSpaceCollapse::Preserve,
                                ),
                            ]);
                            builder.push_text("\n");
                            builder.pop_style_span();
                            text_transformer.word_break(builder);
                        } else {
                            // node.remove_damage(CONSTRUCT_DESCENDENT | CONSTRUCT_FC | CONSTRUCT_BOX);
                            let style = node
                                .primary_styles()
                                .map(|s| stylo_to_parley::style(node.id, &s))
                                .unwrap_or_else(|| parley::TextStyle {
                                    white_space_collapse: WhiteSpaceCollapse::Collapse,
                                    ..Default::default()
                                });

                            // Horizontal margins outside the element's span, borders and
                            // paddings inside it (so its background covers them).
                            let edges = node
                                .primary_styles()
                                .as_deref()
                                .map(|style| inline_edges(style))
                                .unwrap_or_default();
                            push_spacer(builder, edges.start_outer);
                            builder.push_style_span(style);
                            push_spacer(builder, edges.start_inner);
                            let (bidi_start, bidi_end) = node
                                .primary_styles()
                                .as_deref()
                                .map(|style| bidi_controls(style, false))
                                .unwrap_or_default();
                            builder.push_text(bidi_start);

                            if let Some(before_id) = node.before() {
                                build_inline_layout_recursive(
                                    builder,
                                    text_transformer,
                                    nodes,
                                    before_id,
                                    &text_transform,
                                );
                            }

                            for child_id in node.children.iter().copied() {
                                build_inline_layout_recursive(
                                    builder,
                                    text_transformer,
                                    nodes,
                                    child_id,
                                    &text_transform,
                                );
                            }
                            if let Some(after_id) = node.after() {
                                build_inline_layout_recursive(
                                    builder,
                                    text_transformer,
                                    nodes,
                                    after_id,
                                    &text_transform,
                                );
                            }

                            builder.push_text(bidi_end);
                            push_spacer(builder, edges.end_inner);
                            builder.pop_style_span();
                            push_spacer(builder, edges.end_outer);
                        }
                    }
                    // Inline box
                    (_, _) => {
                        if box_kind == InlineBoxKind::InFlow {
                            text_transformer.word_break(builder);
                        }
                        builder.push_inline_box(InlineBox {
                            id: node_id.as_u64(),
                            kind: box_kind,
                            // Overridden by push_inline_box method
                            index: 0,
                            // Width and height are set during layout
                            width: 0.0,
                            height: 0.0,
                            baseline: None,
                            vertical_align: node
                                .primary_styles()
                                .map(|s| stylo_to_parley::vertical_align(&s))
                                .unwrap_or_default(),
                        });
                    }
                };
            }
            NodeData::Text(data) => {
                // node.remove_damage(CONSTRUCT_DESCENDENT | CONSTRUCT_FC | CONSTRUCT_BOX);
                // dbg!(&data.content);

                let text =
                    text_transformer.transform(&data.content, parent_text_transform, builder);
                builder.push_text(text);
            }
            NodeData::Comment { .. } => {
                // node.remove_damage(CONSTRUCT_DESCENDENT | CONSTRUCT_FC | CONSTRUCT_BOX);
            }
            NodeData::Document(_) => unreachable!(),
        }
    }
}

/// The horizontal room an inline element takes before and after its content, in CSS pixels:
/// its margin (outside its box) and its border and padding (inside it), on the inline-start and
/// inline-end sides. Percentages, which resolve against the containing block, count as 0.
#[derive(Clone, Copy, Debug, Default)]
struct InlineEdges {
    start_outer: f32,
    start_inner: f32,
    end_inner: f32,
    end_outer: f32,
}

fn inline_edges(style: &style::properties::ComputedValues) -> InlineEdges {
    use style::computed_values::direction::T as Direction;
    use style::values::computed::{LengthPercentage, Margin};
    let length = |value: &LengthPercentage| value.to_length().map_or(0.0, |length| length.px());
    let margin = |value: &Margin| match value {
        Margin::LengthPercentage(value) => length(value),
        _ => 0.0,
    };
    let margins = style.get_margin();
    let paddings = style.get_padding();
    let borders = style.get_border();
    // A border of style none or hidden is 0 wide, whatever its width says.
    let border = |style: style::values::computed::BorderStyle, width: f32| {
        if style.none_or_hidden() { 0.0 } else { width }
    };
    let left = (
        margin(&margins.margin_left),
        border(borders.border_left_style, borders.border_left_width.0.to_f32_px())
            + length(&paddings.padding_left.0),
    );
    let right = (
        margin(&margins.margin_right),
        border(borders.border_right_style, borders.border_right_width.0.to_f32_px())
            + length(&paddings.padding_right.0),
    );
    let (start, end) = if style.clone_direction() == Direction::Rtl {
        (right, left)
    } else {
        (left, right)
    };
    InlineEdges {
        start_outer: start.0,
        start_inner: start.1,
        end_inner: end.1,
        end_outer: end.0,
    }
}

/// `width` CSS pixels of room on the line: a word joiner (no break, no ink) spaced out by
/// letter-spacing, in the layout text only.
fn push_spacer(builder: &mut TreeBuilder<TextBrush>, width: f32) {
    if width == 0.0 || !width.is_finite() {
        return;
    }
    builder.push_style_modification_span(&[parley::StyleProperty::LetterSpacing(width)]);
    builder.push_text("\u{2060}");
    builder.pop_style_span();
}

// CSS Writing Modes section 2.4.2: translate CSS boundaries into UAX9 controls.
// These belong to the layout text only; the document's source text is unchanged.
fn bidi_controls(
    style: &style::properties::ComputedValues,
    root: bool,
) -> (&'static str, &'static str) {
    use style::computed_values::{direction::T as Direction, unicode_bidi::T as Bidi};
    let rtl = style.clone_direction() == Direction::Rtl;
    let mode = style.clone_unicode_bidi();
    if root {
        return match mode {
            Bidi::BidiOverride | Bidi::IsolateOverride => {
                (if rtl { "\u{202e}" } else { "\u{202d}" }, "\u{202c}")
            }
            _ => ("", ""),
        };
    }
    match mode {
        Bidi::Embed => (if rtl { "\u{202b}" } else { "\u{202a}" }, "\u{202c}"),
        Bidi::Isolate => (if rtl { "\u{2067}" } else { "\u{2066}" }, "\u{2069}"),
        Bidi::BidiOverride => (if rtl { "\u{202e}" } else { "\u{202d}" }, "\u{202c}"),
        Bidi::IsolateOverride => (
            if rtl {
                "\u{2068}\u{202e}"
            } else {
                "\u{2068}\u{202d}"
            },
            "\u{202c}\u{2069}",
        ),
        Bidi::Plaintext => ("\u{2068}", "\u{2069}"),
        _ => ("", ""),
    }
}
