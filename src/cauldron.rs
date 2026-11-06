use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(GameUpdate, add_item_to_cauldron);
}

#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
#[require(Interactable::Cauldron)]
pub struct Cauldron {
    pub items: Vec<Item>,
}

fn add_item_to_cauldron(
    mut messages: MessageReader<PlayerInteraction>,
    mut inventory: Single<&mut Inventory>,
    mut cauldron: Single<&mut Cauldron>,
) {
    let Some(PlayerInteraction {
        kind: Interactable::Cauldron,
        ..
    }) = messages.read().next()
    else {
        return;
    };

    if !inventory.selection.as_ref().is_some_and(|s| {
        s.is_item()
            && !matches!(
                s,
                Selection::Item(Item {
                    kind: ItemKind::Potion,
                    ..
                })
            )
    }) {
        return;
    }

    let Some(Selection::Item(item)) = inventory.selection.take() else {
        return;
    };

    cauldron.items.push(item);
}
