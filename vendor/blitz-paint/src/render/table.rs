//! Shared table-cell visibility rules for the cell and its ancestor backgrounds.
use blitz_dom::{BaseDocument, CopyStyleGetters, Node, node::NodeData};
use style::values::computed::{BorderCollapse, EmptyCells, WhiteSpaceCollapse};
use style::values::specified::box_::DisplayInside;

pub(super) fn hides_empty_cell(doc: &BaseDocument, cell: &Node) -> bool {
    let Some(style) = cell.primary_styles() else {
        return false;
    };
    if style.clone_display().inside() != DisplayInside::TableCell
        || style.clone_border_collapse() != BorderCollapse::Separate
        || style.slow_clone_empty_cells() != EmptyCells::Hide
    {
        return false;
    }
    let whitespace = style.clone_white_space_collapse();
    for id in cell
        .before()
        .into_iter()
        .chain(cell.children.iter().copied())
        .chain(cell.after())
    {
        let Some(child) = doc.get_node(id) else {
            continue;
        };
        match &child.data {
            NodeData::Text(text) => {
                let visible = text.content.chars().any(|c| {
                    if !matches!(c, ' ' | '\t' | '\n' | '\r' | '\u{000c}') {
                        return true;
                    }
                    match whitespace {
                        WhiteSpaceCollapse::Preserve
                        | WhiteSpaceCollapse::PreserveSpaces
                        | WhiteSpaceCollapse::BreakSpaces => true,
                        WhiteSpaceCollapse::PreserveBreaks => matches!(c, '\n' | '\r'),
                        _ => false,
                    }
                });
                if visible {
                    return false;
                }
            }
            NodeData::Element(_) | NodeData::AnonymousBlock(_) => {
                if !child.is_out_of_flow() && child.taffy_display() != taffy::Display::None {
                    return false;
                }
            }
            _ => {}
        }
    }
    true
}
