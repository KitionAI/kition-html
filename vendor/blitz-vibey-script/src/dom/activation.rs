//! Form and disclosure activation shared by pointer and scripted clicks.
use crate::state::DomCtx;
use blitz_dom::{NodeId, local_name};

pub(crate) enum Activation {
    Checked {
        id: NodeId,
        previous: Vec<(NodeId, bool)>,
        changed: bool,
        indeterminate: bool,
    },
    Details {
        id: NodeId,
    },
    Label {
        control: NodeId,
    },
    Reset {
        form: NodeId,
    },
    Suppress,
}
pub(crate) enum Completion {
    Change(NodeId),
    Click(NodeId),
    Reset(NodeId),
}

pub(crate) fn prepare(ctx: &DomCtx, target: NodeId) -> Option<Activation> {
    let mut doc = ctx.doc.borrow_mut();
    let mut next = Some(target);
    while let Some(id) = next {
        let node = doc.get_node(id)?;
        next = node.parent;
        let Some(element) = node.element_data() else {
            continue;
        };
        if node.is_effectively_disabled() {
            return Some(Activation::Suppress);
        }
        if element.name.local == local_name!("summary") {
            let parent = node.parent?;
            let details = doc.get_node(parent)?;
            if details
                .data
                .is_element_with_tag_name(&local_name!("details"))
                && details.children.iter().copied().find(|&id| {
                    doc.get_node(id)
                        .is_some_and(|n| n.data.is_element_with_tag_name(&local_name!("summary")))
                }) == Some(id)
            {
                return Some(Activation::Details { id: parent });
            }
            return None;
        }
        if element.name.local == local_name!("label") {
            return doc
                .label_bound_input_element(id)
                .map(|control| Activation::Label {
                    control: control.id,
                });
        }
        if matches!(element.name.local.as_ref(), "input" | "button") {
            let kind = element
                .attr(local_name!("type"))
                .unwrap_or("")
                .to_ascii_lowercase();
            if kind == "reset" {
                return Some(
                    doc.form_owner(id)
                        .map_or(Activation::Suppress, |form| Activation::Reset { form }),
                );
            }
            if element.name.local != local_name!("input")
                || !matches!(kind.as_str(), "checkbox" | "radio")
            {
                return (kind == "button").then_some(Activation::Suppress);
            }
            let old = element
                .checkbox_input_checked()
                .unwrap_or_else(|| element.has_attr(local_name!("checked")));
            let indeterminate = node.is_indeterminate();
            let ids = if kind == "radio" {
                doc.radio_group(id)
            } else {
                vec![id]
            };
            let previous = ids
                .into_iter()
                .filter_map(|id| {
                    let element = doc.get_node(id)?.element_data()?;
                    Some((
                        id,
                        element
                            .checkbox_input_checked()
                            .unwrap_or_else(|| element.has_attr(local_name!("checked"))),
                    ))
                })
                .collect();
            let checked = kind == "radio" || !old;
            doc.set_input_checked(id, checked);
            if kind == "checkbox" {
                doc.set_input_indeterminate(id, false);
            }
            return Some(Activation::Checked {
                id,
                previous,
                changed: checked != old,
                indeterminate,
            });
        }
        // Interactive descendants have their own activation, never their label's.
        if matches!(
            element.name.local.as_ref(),
            "select" | "textarea" | "a" | "audio" | "video"
        ) {
            return None;
        }
    }
    None
}

pub(crate) fn finish(ctx: &DomCtx, activation: Activation, canceled: bool) -> Option<Completion> {
    let mut doc = ctx.doc.borrow_mut();
    match activation {
        Activation::Checked {
            id,
            previous,
            changed,
            indeterminate,
        } => {
            if canceled {
                for (id, checked) in previous {
                    doc.set_input_checked_direct(id, checked);
                }
                doc.set_input_indeterminate(id, indeterminate);
                None
            } else {
                changed.then_some(Completion::Change(id))
            }
        }
        Activation::Details { id } => {
            if !canceled && doc.get_node(id).is_some() {
                doc.toggle_details_open(id);
            }
            None
        }
        Activation::Label { control } => (!canceled
            && doc
                .get_node(control)
                .is_some_and(|n| !n.is_effectively_disabled()))
        .then_some(Completion::Click(control)),
        Activation::Reset { form } => (!canceled).then_some(Completion::Reset(form)),
        Activation::Suppress => None,
    }
}
