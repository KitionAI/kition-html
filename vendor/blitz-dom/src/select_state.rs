//! Live option selection, independent of source defaults and layout preparation.
use crate::{BaseDocument, NodeId, local_name};
use style_dom::ElementState;

impl BaseDocument {
    pub fn option_select(&self, mut id: NodeId) -> Option<NodeId> {
        while let Some(parent) = self.get_node(id).and_then(|node| node.parent) {
            if self.nodes[parent]
                .data
                .is_element_with_tag_name(&local_name!("select"))
            {
                return Some(parent);
            }
            id = parent;
        }
        None
    }

    pub fn select_options(&self, id: NodeId) -> Vec<NodeId> {
        crate::traversal::TreeTraverser::new_with_root(self, id)
            .skip(1)
            .filter(|&option| {
                self.nodes[option]
                    .data
                    .is_element_with_tag_name(&local_name!("option"))
                    && self.option_select(option) == Some(id)
            })
            .collect()
    }

    pub fn option_selected(&self, id: NodeId) -> bool {
        self.get_node(id)
            .and_then(|node| node.element_data())
            .is_some_and(|element| {
                element
                    .selectedness
                    .unwrap_or_else(|| element.has_attr(local_name!("selected")))
            })
    }

    pub fn selected_options(&self, id: NodeId) -> Vec<NodeId> {
        self.select_options(id)
            .into_iter()
            .filter(|&option| self.option_selected(option))
            .collect()
    }

    pub(crate) fn set_option_selected_direct(&mut self, id: NodeId, selected: bool) {
        let element = self.nodes[id].element_data_mut().unwrap();
        if element.element_state.contains(ElementState::CHECKED) == selected {
            element.selectedness = Some(selected);
            return;
        }
        let unstyled = self.nodes[id].primary_styles().is_none();
        self.snapshot_node_and(id, ElementState::CHECKED, |node| {
            let element = node.element_data_mut().unwrap();
            element.apply_option_selected(selected);
            node.mark_ancestors_dirty();
        });
        // Native select options (and display:none descendants) may never be
        // styled, so Stylo cannot snapshot them. Recascade a styled root to
        // invalidate relational selectors that observe their live state.
        if unstyled && self.style_depends_on_state(ElementState::CHECKED) {
            if let Some(root) = self.try_root_element().map(|node| node.id) {
                if self.nodes[root].primary_styles().is_some() {
                    self.nodes[root].set_restyle_hint(
                        style::invalidation::element::restyle_hints::RestyleHint::restyle_subtree(),
                    );
                }
            }
        }
    }

    /// An explicit select.value/selectedIndex assignment may leave no selection.
    /// Only the chosen option becomes dirty; deselected peers keep their dirtiness.
    pub fn set_select_selection(&mut self, id: NodeId, chosen: Option<NodeId>) {
        for option in self.select_options(id) {
            self.set_option_selected_direct(option, Some(option) == chosen);
        }
        if let Some(chosen) = chosen {
            self.nodes[chosen]
                .element_data_mut()
                .unwrap()
                .dirty_selectedness = true;
        }
    }

    pub fn set_option_selected(&mut self, id: NodeId, selected: bool) {
        self.nodes[id]
            .element_data_mut()
            .unwrap()
            .dirty_selectedness = true;
        self.set_option_selected_direct(id, selected);
        if let Some(owner) = self.option_select(id) {
            self.normalize_select(owner, selected.then_some(id));
        }
    }

    /// Apply the select reset algorithm after a relevant DOM/default mutation.
    /// A selected inserted option wins over existing peers in a single select.
    pub(crate) fn normalize_select(&mut self, id: NodeId, preferred: Option<NodeId>) {
        let options = self.select_options(id);
        if self.nodes[id].data.has_attr(local_name!("multiple")) {
            for option in options {
                self.set_option_selected_direct(option, self.option_selected(option));
            }
            return;
        }
        let chosen = preferred
            .filter(|option| options.contains(option) && self.option_selected(*option))
            .or_else(|| {
                options
                    .iter()
                    .rev()
                    .copied()
                    .find(|&option| self.option_selected(option))
            })
            .or_else(|| {
                let size = self.nodes[id]
                    .attr(local_name!("size"))
                    .and_then(|value| {
                        style::servo::attr::parse_unsigned_integer(value.chars()).ok()
                    })
                    .unwrap_or(0);
                (size <= 1)
                    .then(|| {
                        options
                            .iter()
                            .copied()
                            .find(|&option| !self.nodes[option].is_effectively_disabled())
                    })
                    .flatten()
            });
        for option in options {
            self.set_option_selected_direct(option, Some(option) == chosen);
        }
    }

    /// Text changes inside an option do not run the select reset algorithm.
    pub(crate) fn select_for_changed_subtree(&self, root: NodeId) -> Option<NodeId> {
        let owner = self.option_select(root)?;
        crate::traversal::TreeTraverser::new_with_root(self, root)
            .any(|id| {
                self.nodes[id]
                    .data
                    .is_element_with_tag_name(&local_name!("option"))
                    && self.option_select(id) == Some(owner)
            })
            .then_some(owner)
    }

    pub(crate) fn inserted_select_subtrees(&mut self, roots: &[NodeId]) {
        let mut updates = Vec::<(NodeId, Option<NodeId>)>::new();
        // Snapshot all incoming selections before resolving any single-select
        // conflicts. A DocumentFragment can contain several selected options.
        for &root in roots {
            let Some(owner) = self.select_for_changed_subtree(root) else {
                continue;
            };
            let selected = crate::traversal::TreeTraverser::new_with_root(self, root)
                .filter(|&id| {
                    self.nodes[id]
                        .data
                        .is_element_with_tag_name(&local_name!("option"))
                        && self.option_select(id) == Some(owner)
                        && self.option_selected(id)
                })
                .last();
            if let Some((_, preferred)) = updates.iter_mut().find(|(id, _)| *id == owner) {
                if selected.is_some() {
                    *preferred = selected;
                }
            } else {
                updates.push((owner, selected));
            }
        }
        for (owner, preferred) in updates {
            self.normalize_select(owner, preferred);
        }
    }

    pub(crate) fn select_attribute_changed(&mut self, id: NodeId, name: &str) {
        if self.nodes[id]
            .data
            .is_element_with_tag_name(&local_name!("option"))
            && name == "selected"
        {
            let element = self.nodes[id].element_data().unwrap();
            if !element.dirty_selectedness {
                let selected = element.has_attr(local_name!("selected"));
                self.set_option_selected_direct(id, selected);
                if let Some(owner) = self.option_select(id) {
                    self.normalize_select(owner, selected.then_some(id));
                }
            }
        } else if self.nodes[id]
            .data
            .is_element_with_tag_name(&local_name!("select"))
            && matches!(name, "multiple" | "size")
        {
            // Switching from multiple to single retains the first selected option.
            let first = self.selected_options(id).first().copied();
            self.normalize_select(id, first);
        }
    }

    pub(crate) fn reset_select(&mut self, id: NodeId) {
        for option in self.select_options(id) {
            self.nodes[option]
                .element_data_mut()
                .unwrap()
                .dirty_selectedness = false;
            let selected = self.nodes[option].data.has_attr(local_name!("selected"));
            self.set_option_selected_direct(option, selected);
        }
        self.normalize_select(id, None);
    }
}
