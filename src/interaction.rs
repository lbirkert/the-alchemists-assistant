use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_message::<PlayerInteraction>();
}

#[derive(Message, Debug)]
pub struct PlayerInteraction {
    pub entity: Entity,
    pub kind: Interactable,
}

#[derive(Component, Reflect, Debug, Clone, Copy)]
#[reflect(Component)]
pub enum Interactable {
    Item(Item),
    PlaceableLocation,
    Discardable,
    Source { item: Item },
    Drawer,
    Mortar,
}
