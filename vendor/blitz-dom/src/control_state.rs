//! Form control state that is distinct from serialized default attributes.
use crate::layout::damage::ALL_DAMAGE;
use crate::{BaseDocument, NodeId, local_name};
use style_dom::ElementState;

impl BaseDocument {
    /// Radio peers share a nonempty name, form owner, and tree root.
    pub fn radio_group(&self, node_id: NodeId) -> Vec<NodeId> {
        let Some(node) = self.get_node(node_id) else {
            return Vec::new();
        };
        let Some(element) = node.element_data() else {
            return Vec::new();
        };
        let Some(name) = element
            .attr(local_name!("name"))
            .filter(|name| !name.is_empty())
        else {
            return vec![node_id];
        };
        let root = |mut id| {
            while let Some(parent) = self.get_node(id).and_then(|node| node.parent) {
                id = parent;
            }
            id
        };
        let owner = self.form_owner(node_id);
        let tree = root(node_id);
        self.nodes
            .iter()
            .filter_map(|(id, node)| {
                let element = node.element_data()?;
                (element.name.local == local_name!("input")
                    && element
                        .attr(local_name!("type"))
                        .is_some_and(|kind| kind.eq_ignore_ascii_case("radio"))
                    && element.attr(local_name!("name")) == Some(name)
                    && self.form_owner(id) == owner
                    && root(id) == tree)
                    .then_some(id)
            })
            .collect()
    }

    pub fn set_input_checked(&mut self, node_id: NodeId, checked: bool) {
        let radio = self
            .get_node(node_id)
            .and_then(|n| n.element_data())
            .is_some_and(|e| {
                e.name.local == local_name!("input")
                    && e.attr(local_name!("type"))
                        .is_some_and(|kind| kind.eq_ignore_ascii_case("radio"))
            });
        if checked && radio {
            for peer in self.radio_group(node_id) {
                if peer != node_id {
                    self.set_input_checked_direct(peer, false);
                }
            }
        }
        self.set_input_checked_direct(node_id, checked);
    }

    pub fn set_input_checked_direct(&mut self, node_id: NodeId, checked: bool) {
        if self.get_node(node_id).is_none() {
            return;
        }
        self.snapshot_node_and(node_id, ElementState::CHECKED, |node| {
            if let Some(element) = node.element_data_mut() {
                element.set_checkbox_input_checked(checked);
            }
            node.mark_ancestors_dirty();
        });
    }

    pub fn set_input_value(&mut self, node_id: NodeId, value: &str) {
        let Some(node) = self.nodes.get_mut(node_id) else {
            return;
        };
        let Some(element) = node.element_data_mut() else {
            return;
        };
        element.dirty_value = Some(value.to_owned());
        if let Some(input) = element.text_input_data_mut() {
            input.set_text(
                &mut self.font_ctx.lock().unwrap(),
                &mut self.layout_ctx,
                value,
            );
        }
        node.insert_damage(ALL_DAMAGE);
        node.mark_ancestors_dirty();
    }
}

impl BaseDocument {
    /// Resolve the current owner from the DOM, including detached and moved controls.
    pub fn form_owner(&self, node_id: NodeId) -> Option<NodeId> {
        let node = self.get_node(node_id)?;
        let element = node.element_data()?;
        if let Some(owner) = element.attr(local_name!("form")) {
            if !node.flags.is_in_document() {
                return None;
            }
            return self.get_element_by_id(owner).filter(|&id| {
                self.nodes[id]
                    .data
                    .is_element_with_tag_name(&local_name!("form"))
            });
        }
        let mut parent = node.parent;
        while let Some(id) = parent {
            let node = self.get_node(id)?;
            if node.data.is_element_with_tag_name(&local_name!("form")) {
                return Some(id);
            }
            parent = node.parent;
        }
        None
    }

    /// Reset live control state while keeping source defaults intact.
    pub fn reset_form_controls(&mut self, form_id: NodeId) {
        let mut root = form_id;
        while let Some(parent) = self.nodes[root].parent {
            root = parent;
        }
        let controls: Vec<_> = crate::traversal::TreeTraverser::new_with_root(self, root)
            .filter(|&id| self.form_owner(id) == Some(form_id))
            .collect();
        for id in controls {
            let node = &self.nodes[id];
            let Some(element) = node.element_data() else {
                continue;
            };
            match element.name.local.as_ref() {
                "input" | "textarea" => {
                    let checked = element.has_attr(local_name!("checked"));
                    let value = if element.name.local == local_name!("textarea") {
                        node.text_content()
                    } else {
                        element.attr(local_name!("value")).unwrap_or("").to_owned()
                    };
                    self.set_input_checked(id, checked);
                    self.set_input_value(id, &value);
                    let element = self.nodes[id].element_data_mut().unwrap();
                    element.checkedness = None;
                    element.dirty_value = None;
                }
                "select" => self.reset_select(id),
                _ => {}
            }
        }
    }

    pub fn set_input_indeterminate(&mut self, id: NodeId, value: bool) {
        self.snapshot_node_and(id, ElementState::INDETERMINATE, |node| {
            if let Some(element) = node.element_data_mut() {
                element
                    .element_state
                    .set(ElementState::INDETERMINATE, value);
            }
            node.mark_ancestors_dirty();
        });
    }
}

impl crate::Node {
    pub fn is_indeterminate(&self) -> bool {
        if self.data.is_element_with_tag_name(&local_name!("progress")) {
            return !self.data.has_attr(local_name!("value"));
        }
        self.element_data()
            .is_some_and(|e| e.element_state.contains(ElementState::INDETERMINATE))
    }

    /// Disabled fieldsets exclude descendants of their first legend.
    pub fn is_effectively_disabled(&self) -> bool {
        let Some(element) = self.element_data() else {
            return false;
        };
        if !element.can_be_disabled() {
            return false;
        }
        if element.has_attr(local_name!("disabled")) {
            return true;
        }
        if element.name.local == local_name!("option") {
            return self.parent.is_some_and(|id| {
                let parent = self.with(id);
                parent
                    .data
                    .is_element_with_tag_name(&local_name!("optgroup"))
                    && parent.data.has_attr(local_name!("disabled"))
            });
        }
        if element.name.local == local_name!("optgroup") {
            return false;
        }
        let mut descendant = self.id;
        let mut parent = self.parent;
        while let Some(id) = parent {
            let node = self.with(id);
            if node.data.is_element_with_tag_name(&local_name!("fieldset"))
                && node.data.has_attr(local_name!("disabled"))
            {
                let legend = node.children.iter().copied().find(|&id| {
                    self.with(id)
                        .data
                        .is_element_with_tag_name(&local_name!("legend"))
                });
                if legend != Some(descendant) {
                    return true;
                }
            }
            descendant = id;
            parent = node.parent;
        }
        false
    }
}
