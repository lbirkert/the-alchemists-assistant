use bevy::picking::backend::HitData;

use crate::{item::ITEM_SIZE, prelude::*, scene::NpcTable};

pub fn plugin(app: &mut App) {
    app.add_systems(GameUpdate, fill_potion);
    app.add_systems(GameUpdate, label_potion);
    app.add_systems(GameUpdate, submit_potion);
}

// a potion crafted by a user that might have a sticky note attached
#[derive(Reflect, Clone, Debug, Default, PartialEq, Eq, Hash)]
#[reflect(Default)]
pub struct PotionData {
    pub items: Vec<Item>,
    pub recipe: Option<Recipe>,
    // TODO: type: round, square, etc...
}

impl PotionData {
    pub fn can_validate(&self) -> bool {
        self.recipe.is_some()
    }

    pub fn validate(&self) -> Option<bool> {
        Some(self.items == self.recipe.as_ref()?.items())
    }

    pub fn color(&self) -> Color {
        crate::utils::items_to_color(&self.items)
    }
}

fn fill_potion(
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

    if cauldron.is_empty() {
        return;
    }

    if !matches!(
        inventory.selection,
        Some(Selection::Item(Item {
            kind: ItemKind::Potion,
            potion: None,
            ..
        }))
    ) {
        return;
    }

    let potion = PotionData {
        items: cauldron.items.clone(),
        recipe: None,
    };
    cauldron.items.truncate(0);
    inventory.selection.replace(Selection::Item(Item {
        kind: ItemKind::Potion,
        potion: Some(potion),
        ..Default::default()
    }));
}

fn label_potion(
    mut commands: Commands,
    mut messages: MessageReader<PlayerInteraction>,
    mut inventory: Single<&mut Inventory>,
) {
    let Some(PlayerInteraction {
        entity,
        kind: Interactable::Item(item),
        ..
    }) = messages.read().next()
    else {
        return;
    };

    let mut item = item.clone();
    let Item {
        potion: Some(ref mut potion @ PotionData { recipe: None, .. }),
        ..
    } = item
    else {
        return;
    };

    if !matches!(inventory.selection, Some(Selection::StickyNote(..))) {
        return;
    };

    let Some(Selection::StickyNote(sticky_note)) = inventory.selection.take() else {
        panic!();
    };

    potion.recipe = sticky_note.recipe.clone();

    // update potion (so that sticky note shows)
    commands
        .entity(*entity)
        .insert((Interactable::Item(item.clone()), item));
}

fn submit_potion(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut messages: MessageReader<PlayerInteraction>,
    mut inventory: Single<&mut Inventory>,
    mut npc_table: Single<&mut NpcTable>,
) {
    let Some(PlayerInteraction {
        kind: Interactable::NpcTable,
        hit:
            HitData {
                position: Some(position),
                normal: Some(normal),
                ..
            },
        ..
    }) = messages.read().next()
    else {
        return;
    };

    if !matches!(
        inventory.selection,
        Some(Selection::Item(Item {
            potion: Some(PotionData {
                recipe: Some(..),
                ..
            }),
            ..
        }))
    ) {
        return;
    };

    let Some(Selection::Item(item)) = inventory.selection.take() else {
        panic!();
    };

    // TODO: we should maybe move the potion to a predefined grid instead of taking hit position
    let scene = item.clone().scene(
        &asset_server,
        Transform::from_translation(*position + *normal * (ITEM_SIZE / 2.0)),
    );

    let potion_entity = commands.spawn_scene(scene).id();
    npc_table
        .potions
        .push((potion_entity, item.potion.unwrap()));
}
