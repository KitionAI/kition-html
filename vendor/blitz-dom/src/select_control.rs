//! Selected labels and intrinsic measurements for collapsed HTML select controls.
use crate::{
    BaseDocument, NodeId, local_name,
    node::{SpecialElementData, TextLayout},
    stylo_to_parley,
};
use parley::{Alignment, AlignmentOptions, LineHeight, WhiteSpaceCollapse};
use style::values::computed::Appearance;
use stylo_taffy::CopyStyleGetters;

#[derive(Clone, Debug)]
pub struct SelectControl {
    pub label: TextLayout,
    pub width: f32,
    pub height: f32,
    pub text_height: f32,
    pub baseline: f32,
    pub native: bool,
    pub rtl: bool,
    labels: Vec<(String, bool)>,
}

impl BaseDocument {
    fn option_label(&self, id: NodeId) -> String {
        let node = &self.nodes[id];
        node.attr(local_name!("label"))
            .map(str::to_owned)
            .unwrap_or_else(|| {
                node.text_content()
                    .split_ascii_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ")
            })
    }

    fn select_labels(&self, id: NodeId) -> (String, Vec<(String, bool)>) {
        let selected = self
            .selected_options(id)
            .first()
            .map(|&option| self.option_label(option))
            .unwrap_or_default();
        (
            selected,
            self.select_options(id)
                .into_iter()
                .map(|option| {
                    let grouped = self.nodes[option].parent.is_some_and(|parent| {
                        self.nodes[parent]
                            .data
                            .is_element_with_tag_name(&local_name!("optgroup"))
                    });
                    (self.option_label(option), grouped)
                })
                .collect(),
        )
    }

    pub(crate) fn refresh_select_labels(&mut self) {
        let changed: Vec<_> = self
            .nodes
            .iter()
            .filter_map(|(id, node)| {
                let SpecialElementData::Select(control) = &node.element_data()?.special_data else {
                    return None;
                };
                let (label, labels) = self.select_labels(id);
                (label != control.label.text || labels != control.labels).then_some(id)
            })
            .collect();
        for id in changed {
            self.nodes[id].insert_damage(crate::layout::damage::ALL_DAMAGE);
        }
    }

    pub(crate) fn prepare_select(&mut self, id: NodeId) {
        let (selected, labels) = self.select_labels(id);
        let node = &self.nodes[id];
        let Some(style) = node.primary_styles() else {
            return;
        };
        let native = style.get_box().appearance != Appearance::None;
        let rtl = style.clone_direction() == style::computed_values::direction::T::Rtl;
        let mut text_style = stylo_to_parley::style(id, &style);
        text_style.line_height = LineHeight::NORMAL;
        text_style.white_space_collapse = WhiteSpaceCollapse::Preserve;
        let scale = self.viewport.scale();
        let mut fonts = self.font_ctx.lock().unwrap();
        let mut grouped_style = text_style.clone();
        // Optgroup options use normal weight for menu sizing, even when the
        // collapsed selected label inherits a bold select font.
        grouped_style.font_weight = parley::FontWeight::NORMAL;
        let mut shape = |text: &str, grouped: bool| {
            let mut builder = self.layout_ctx.tree_builder(
                &mut fonts,
                scale,
                true,
                if grouped { &grouped_style } else { &text_style },
            );
            builder.set_base_direction(if rtl {
                parley::BaseDirection::Rtl
            } else {
                parley::BaseDirection::Ltr
            });
            builder.push_text(text);
            let mut layout = builder.build().0;
            layout.break_all_lines(None);
            layout.align(Alignment::Start, AlignmentOptions::default());
            layout
        };
        let mut width = 0_f32;
        for (label, grouped) in &labels {
            // Group labels do not set the collapsed control's width. Grouped
            // options reserve four shaped spaces, including with appearance:none.
            let indented;
            let text = if *grouped {
                indented = format!("    {label}");
                &indented
            } else {
                label
            };
            width = width.max(shape(text, *grouped).full_width() / scale);
        }
        let layout = shape(&selected, false);
        let metrics = shape(" ", false);
        let height = metrics
            .lines()
            .next()
            .and_then(|line| line.runs().next())
            .map(|run| {
                (run.font_metrics().ascent / scale).round()
                    + (run.font_metrics().descent / scale).round()
            })
            .unwrap_or(text_style.font_size);
        let baseline = metrics
            .lines()
            .next()
            .map_or(height, |line| line.metrics().baseline / scale);
        drop(fonts);
        drop(style);
        let label = TextLayout {
            text: selected,
            layout,
            ..Default::default()
        };
        self.nodes[id].element_data_mut().unwrap().special_data =
            SpecialElementData::Select(Box::new(SelectControl {
                label,
                width: width.ceil() + if native { 20. } else { 0. },
                height: height + if native { 2. } else { 0. },
                text_height: height,
                baseline,
                native,
                rtl,
                labels,
            }));
    }
}
