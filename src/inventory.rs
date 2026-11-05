use bevy::picking::backend::HitData;

use crate::{item::ITEM_SIZE, prelude::*};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(GameUpdate, pickup_item);
    app.add_systems(GameUpdate, pickup_item_from_source);
    app.add_systems(GameUpdate, place_item);
    app.add_systems(GameUpdate, discard_item);
}

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

fn pickup_item_from_source(
    mut messages: MessageReader<PlayerInteraction>,
    mut inventory: Single<&mut Inventory>,
) {
    let Some(PlayerInteraction {
        kind: Interactable::Source { item: source },
        ..
    }) = messages.read().next()
    else {
        return;
    };

    if inventory.selection.is_some() {
        return;
    }

    inventory.selection = Some(Selection::Item(*source));
}

fn pickup_item(
    mut messages: MessageReader<PlayerInteraction>,
    mut commands: Commands,
    assets_server: Res<AssetServer>,
    mut inventory: Single<&mut Inventory>,
    transforms: Query<&Transform>,
) {
    let Some(PlayerInteraction {
        entity,
        kind: Interactable::Item(item),
        ..
    }) = messages.read().next()
    else {
        return;
    };

    if !inventory.selection.as_ref().is_none_or(|s| s.is_item()) {
        return;
    }

    if let Ok(item_transform) = transforms.get(*entity) {
        let old_item = inventory.selection.replace(Selection::Item(*item));
        if let Some(Selection::Item(old_item)) = old_item {
            let item_scene =
                old_item.physics_scene(&assets_server, item_transform.translation, Vec3::ZERO);
            commands.spawn_scene(item_scene);
            let item_scene =
                old_item.physics_scene(&assets_server, item_transform.translation, Vec3::ZERO);
            commands.spawn_scene(item_scene);
        }
        // destroy old item
        commands.entity(*entity).despawn();
    }
}

fn place_item(
    mut messages: MessageReader<PlayerInteraction>,
    mut commands: Commands,
    assets_server: Res<AssetServer>,
    mut inventory: Single<&mut Inventory>,
) {
    let Some(PlayerInteraction {
        kind: Interactable::ItemPlaceable,
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

    if !inventory.selection.as_ref().is_some_and(|s| s.is_item()) {
        return;
    }

    let Some(Selection::Item(item)) = inventory.selection.take() else {
        return;
    };

    let item_scene = item.physics_scene(
        &assets_server,
        position + normal * ITEM_SIZE / 2.0,
        Vec3::ZERO,
    );
    commands.spawn_scene(item_scene);
}

fn discard_item(
    mut messages: MessageReader<PlayerInteraction>,
    mut inventory: Single<&mut Inventory>,
) {
    let Some(PlayerInteraction {
        kind: Interactable::ItemDiscardable,
        ..
    }) = messages.read().next()
    else {
        return;
    };

    if !inventory.selection.as_ref().is_some_and(|s| s.is_item()) {
        return;
    }

    inventory.selection = None;
}
