use blitz_traits::node_id::NodeId;
use markup5ever::local_name;
use parley::FontFamily;
use style::computed_values::list_style_type::T as ListStyleType;
use style::{
    computed_values::list_style_position::T as ListStylePosition, counter_style::CounterStyle,
};
use stylo_taffy::CopyStyleGetters;

use crate::{
    BaseDocument,
    node::{ListItemLayout, ListItemLayoutPosition, Marker},
    stylo_to_parley,
};

pub(super) const BULLET_FONT_FAMILY: &str = "Bullet, monospace, sans-serif";

pub(super) fn collect_list_item_children(
    doc: &mut BaseDocument,
    index: &mut i64,
    reversed: bool,
    node_id: NodeId,
) {
    let children = doc.nodes[node_id].children.clone();
    for child in children.into_iter() {
        let node = &doc.nodes[child];
        if node
            .primary_styles()
            .is_some_and(|s| s.get_box().display.is_list_item())
        {
            if let Some(value) = node
                .element_data()
                .and_then(|el| el.attr_parsed::<i32>(local_name!("value")))
            {
                *index = i64::from(value) - 1;
            }
        }
        if let Some(layout) = node_list_item_child(doc, child, *index) {
            let node = &mut doc.nodes[child];
            node.element_data_mut().unwrap().list_item_data = Some(Box::new(layout));
            *index = index.saturating_add(if reversed { -1 } else { 1 });
            collect_list_item_children(doc, index, reversed, child);
        } else {
            // Unset marker in case it was previously set
            let node = &mut doc.nodes[child];
            if let Some(element_data) = node.element_data_mut() {
                element_data.list_item_data = None;
            }
        }
    }
}

// Return a child node which is of display: list-item
fn node_list_item_child(
    doc: &mut BaseDocument,
    child_id: NodeId,
    index: i64,
) -> Option<ListItemLayout> {
    let node = &doc.nodes[child_id];

    // We only care about elements with display: list-item (li's have this automatically)
    if !node
        .primary_styles()
        .is_some_and(|style| style.get_box().display.is_list_item())
    {
        return None;
    }

    //Break on container elements when already in a list
    if node
        .element_data()
        .map(|element_data| {
            matches!(
                element_data.name.local,
                local_name!("ol") | local_name!("ul"),
            )
        })
        .unwrap_or(false)
    {
        return None;
    };

    let styles = node.primary_styles().unwrap();
    let list_style_type = styles.get_list_style_type();
    let list_style_position = styles.clone_list_style_position();
    let marker = marker_for_style(list_style_type, index)?;

    let position = match list_style_position {
        ListStylePosition::Inside => ListItemLayoutPosition::Inside,
        ListStylePosition::Outside => {
            let mut parley_style = textual_marker_style(&stylo_to_parley::style(child_id, &styles));
            // A marker's suffix space separates it from the principal box. Preserve
            // that advance when shaping the standalone outside-marker layout.
            parley_style.white_space_collapse = parley::WhiteSpaceCollapse::Preserve;

            // Create a parley tree builder
            let mut font_ctx = doc.font_ctx.lock().unwrap();
            let geometric = symbol_marker_style(
                &mut doc.layout_ctx,
                &mut font_ctx,
                &parley_style,
                &marker,
                false,
                doc.viewport.scale(),
            );
            if let Some(style) = geometric {
                parley_style = style;
            } else if let Some(font_family) = font_for_bullet_style(list_style_type) {
                parley_style.font_family = font_family;
            }
            let mut builder = doc.layout_ctx.tree_builder(
                &mut font_ctx,
                doc.viewport.scale(),
                true,
                &parley_style,
            );

            match &marker {
                Marker::Char(char) => {
                    if parley_style.brush.marker.is_some() {
                        builder.push_text("\u{a0}");
                    } else {
                        let mut buf = [0u8; 4];
                        builder.push_text(char.encode_utf8(&mut buf));
                    }
                }
                Marker::String(str) => builder.push_text(str),
            };

            let mut layout = builder.build().0;
            let width = layout.calculate_content_widths().max;
            layout.break_all_lines(Some(width));

            ListItemLayoutPosition::Outside(Box::new(layout))
        }
    };

    Some(ListItemLayout { marker, position })
}

/// The UA marker style uses tabular figures, including the font's figure
/// punctuation. Explicit low-level features remain later in the feature list.
pub(super) fn textual_marker_style(
    style: &parley::TextStyle<'static, 'static, crate::node::TextBrush>,
) -> parley::TextStyle<'static, 'static, crate::node::TextBrush> {
    let mut marker_style = style.clone();
    if let parley::FontFeatures::List(features) = &mut marker_style.font_features {
        features.to_mut().insert(
            0,
            parley::FontFeature {
                tag: parley::setting::Tag::from_bytes(*b"tnum"),
                value: 1,
            },
        );
    }
    marker_style
}

/// Shape a placeholder with the principal font, reserving the symbol's actual
/// advance without introducing fallback-font ascent or descent into the line.
pub(super) fn symbol_marker_style(
    layout_ctx: &mut parley::LayoutContext<crate::node::TextBrush>,
    font_ctx: &mut parley::FontContext,
    style: &parley::TextStyle<'static, 'static, crate::node::TextBrush>,
    marker: &Marker,
    inside: bool,
    scale: f32,
) -> Option<parley::TextStyle<'static, 'static, crate::node::TextBrush>> {
    let Marker::Char(symbol @ ('\u{2022}' | '\u{25e6}' | '\u{25aa}' | '\u{25be}' | '\u{25b8}')) =
        marker
    else {
        return None;
    };
    let mut marker_style = style.clone();
    marker_style.letter_spacing = 0.0;
    marker_style.word_spacing = 0.0;
    let mut builder = layout_ctx.tree_builder(font_ctx, scale, true, &marker_style);
    builder.push_text("\u{a0}");
    let mut sample = builder.build().0;
    sample.break_all_lines(None);
    let line = sample.lines().next()?;
    let run = line.runs().next()?;
    let ascent = (run.font_metrics().ascent / scale).round();
    let font_size = style.font_size;
    let disclosure = matches!(symbol, '\u{25be}' | '\u{25b8}');
    let width = if disclosure {
        (font_size * 0.66 * 64.0).floor() / 64.0
    } else {
        (((ascent * 2.0 / 3.0).floor() + 1.0) / 2.0).floor() + 2.0
    };
    let margin = if !inside {
        0.0
    } else if disclosure {
        (font_size * 0.4 * 64.0).floor() / 64.0
    } else {
        font_size - 1.0
    };
    marker_style.letter_spacing = width + margin - sample.full_width() / scale;
    marker_style.brush.marker = Some(*symbol);
    Some(marker_style)
}

// Determine the marker to render for a given list style type
fn marker_for_style(list_style_type: &ListStyleType, index: i64) -> Option<Marker> {
    Some(match &list_style_type.0 {
        CounterStyle::None => return None,
        CounterStyle::Name(name) => match &*name.0 {
            "decimal"
            | "decimal-leading-zero"
            | "lower-roman"
            | "upper-roman"
            | "lower-alpha"
            | "upper-alpha"
            | "lower-latin"
            | "upper-latin" => Marker::String(format!(
                "{}. ",
                super::counters::format_counter(
                    index
                        .saturating_add(1)
                        .clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32,
                    &list_style_type.0
                )
            )),
            "disc" => Marker::Char('•'),
            "circle" => Marker::Char('◦'),
            "square" => Marker::Char('▪'),
            "disclosure-open" => Marker::Char('▾'),
            "disclosure-closed" => Marker::Char('▸'),
            _ => Marker::Char('□'),
        },
        CounterStyle::String(atom_string) => Marker::String(atom_string.as_ref().to_string()),
        CounterStyle::Symbols { symbols, .. } => {
            // Use symbols from counter-style definition
            let syms = &symbols.0;
            let len = syms.len();
            if len == 0 {
                Marker::Char('•')
            } else {
                // Cycle through symbols based on index: 1st, 2nd, 1st, 2nd, etc.
                let idx = if len > 1 {
                    index.rem_euclid(len as i64) as usize
                } else {
                    0
                };
                // Symbol is String or Ident variant, extract the string value
                let sym = &syms[idx];
                let marker_str = match sym {
                    style::counter_style::Symbol::String(s) => s.to_string(),
                    style::counter_style::Symbol::Ident(id) => id.0.to_string(),
                };
                Marker::String(marker_str)
            }
        }
    })
}

// Override the font to our specific bullet font when rendering bullets
fn font_for_bullet_style(list_style_type: &ListStyleType) -> Option<FontFamily<'static>> {
    if list_style_type.0.is_bullet() {
        return Some(BULLET_FONT_FAMILY.into());
    }

    None
}

#[cfg(test)]
mod tests {
    use crate::node::Marker;
    use style::{
        Atom,
        counter_style::CounterStyle,
        values::{CustomIdent, computed::ListStyleType},
    };

    use super::marker_for_style;

    fn list_style(s: &str) -> ListStyleType {
        ListStyleType(CounterStyle::Name(CustomIdent(Atom::from(s))))
    }

    #[test]
    fn test_marker_for_disc() {
        let result = marker_for_style(&ListStyleType::disc(), 0);
        assert_eq!(result, Some(Marker::Char('•')));
    }

    #[test]
    fn test_marker_for_decimal() {
        let result_1 = marker_for_style(&list_style("decimal"), 0);
        let result_2 = marker_for_style(&list_style("decimal"), 1);
        assert_eq!(result_1, Some(Marker::String("1. ".to_string())));
        assert_eq!(result_2, Some(Marker::String("2. ".to_string())));
    }

    #[test]
    fn test_marker_for_lower_alpha() {
        let result_1 = marker_for_style(&list_style("lower-alpha"), 0);
        let result_2 = marker_for_style(&list_style("lower-alpha"), 1);
        let result_extended_1 = marker_for_style(&list_style("lower-alpha"), 26);
        let result_extended_2 = marker_for_style(&list_style("lower-alpha"), 27);
        assert_eq!(result_1, Some(Marker::String("a. ".to_string())));
        assert_eq!(result_2, Some(Marker::String("b. ".to_string())));
        assert_eq!(result_extended_1, Some(Marker::String("aa. ".to_string())));
        assert_eq!(result_extended_2, Some(Marker::String("ab. ".to_string())));
    }

    #[test]
    fn test_marker_for_upper_alpha() {
        let result_1 = marker_for_style(&list_style("upper-alpha"), 0);
        let result_2 = marker_for_style(&list_style("upper-alpha"), 1);
        let result_extended_1 = marker_for_style(&list_style("upper-alpha"), 26);
        let result_extended_2 = marker_for_style(&list_style("upper-alpha"), 27);
        assert_eq!(result_1, Some(Marker::String("A. ".to_string())));
        assert_eq!(result_2, Some(Marker::String("B. ".to_string())));
        assert_eq!(result_extended_1, Some(Marker::String("AA. ".to_string())));
        assert_eq!(result_extended_2, Some(Marker::String("AB. ".to_string())));
    }
}
