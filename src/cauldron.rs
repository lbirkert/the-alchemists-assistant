use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(GameUpdate, add_item_to_cauldron);
    app.add_systems(GamePostUpdate, update_cauldron_color);
}

#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
#[require(Interactable::Cauldron, LiquidController {
    is_visible: false,
    color: Color::srgb(1.0, 0.0, 0.0),
})]
pub struct Cauldron {
    pub items: Vec<Item>,
}

impl Cauldron {
    pub fn color(&self) -> Color {
        crate::utils::items_to_color(&self.items)
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
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

fn update_cauldron_color(mut query: Single<(&mut LiquidController, &Cauldron), Changed<Cauldron>>) {
    let (ref mut liquid_controller, ref mut cauldron) = *query;

    // update liquid color
    liquid_controller.is_visible = !cauldron.items.is_empty();
    liquid_controller.color = cauldron.color();
}
