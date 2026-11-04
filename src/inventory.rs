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
    pub item: Option<Item>,
    // TODO: item_lh and item_rh
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

    if inventory.item.is_some() {
        return;
    }

    inventory.item = Some(*source);
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

    if let Ok(item_transform) = transforms.get(*entity) {
        let old_item = inventory.item.replace(*item);
        if let Some(old_item) = old_item {
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
        kind: Interactable::PlaceableLocation,
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

    let Some(item) = inventory.item.take() else {
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
        kind: Interactable::Discardable,
        ..
    }) = messages.read().next()
    else {
        return;
    };

    inventory.item = None;
}
