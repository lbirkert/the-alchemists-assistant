use crate::prelude::*;

#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect(Component)]
pub struct Inventory {
    pub selection: Option<Selection>,
    // TODO: item_lh and item_rh
}

#[derive(Reflect, Debug, Clone)]
pub enum Selection {
    Item(Item),
    StickyNote(StickyNote),
}

impl Selection {
    pub fn is_item(&self) -> bool {
        matches!(self, Selection::Item(..))
    }

    pub fn is_sticky_node(&self) -> bool {
        matches!(self, Selection::StickyNote(..))
    }
}

impl std::fmt::Display for Selection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Selection::Item(item) => write!(f, "{}", item),
            Selection::StickyNote(..) => write!(f, "Sticky Note"),
        }
    }
}
