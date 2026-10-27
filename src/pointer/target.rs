use crate::display::Position;
use crate::panel::{Composition, Panel};
use crate::window::ViewId;

/// Current surface ownership, independent of any optional text destination.
#[derive(Clone, Copy)]
pub(crate) enum Target {
    Tree(Panel),
    Editing { view: ViewId, panel: Panel },
}

impl Target {
    pub(crate) fn resolve(composition: &Composition, position: Position) -> Option<Self> {
        if let Some(panel) = composition.tree()
            && panel.outer().local(position).is_some()
        {
            return Some(Self::Tree(panel));
        }
        composition
            .panels()
            .iter()
            .find(|(_, panel)| panel.outer().local(position).is_some())
            .map(|(view, panel)| Self::Editing {
                view: *view,
                panel: *panel,
            })
    }
}
