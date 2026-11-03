use bevy::picking::backend::HitData;

use crate::{
    item::{ITEM_SIZE, ItemsSource},
    prelude::*,
};

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
    input: Res<ButtonInput<MouseButton>>,
    mut inventory: Single<&mut Inventory>,
    pointer: Single<&PointerInteraction, With<PlayerCursor>>,
    items_sources: Query<&ItemsSource>,
) {
    if !input.just_pressed(MouseButton::Right) {
        return;
    }

    if let Some((entity, _hit)) = pointer.get_nearest_hit() {
        let Ok(items_source) = items_sources.get(*entity) else {
            return;
        };

        if inventory.item.is_some() {
            return;
            // TODO: also display a message here?
        }

        inventory.item = Some(items_source.item);
    }
}

fn pickup_item(
    mut commands: Commands,
    assets_server: Res<AssetServer>,
    input: Res<ButtonInput<MouseButton>>,
    mut inventory: Single<&mut Inventory>,
    pointer: Single<&PointerInteraction, With<PlayerCursor>>,
    query: Query<(Option<&ChildOf>, Has<Item>)>,
    item_query: Query<(&Item, &Transform)>,
) {
    if !input.just_pressed(MouseButton::Right) {
        return;
    }

    if let Some((mesh_entity, _hit)) = pointer.get_nearest_hit()
        && let Some(entity) = crate::utils::find(*mesh_entity, &query)
        && let Ok((item, item_transform)) = item_query.get(entity)
    {
        let old_item = inventory.item.replace(*item);
        if let Some(old_item) = old_item {
            let item_scene =
                old_item.physics_scene(&assets_server, item_transform.translation, Vec3::ZERO);
            commands.spawn_scene(item_scene);
        }

        // destroy old item
        commands.entity(entity).despawn();
    }
}

fn place_item(
    mut commands: Commands,
    assets_server: Res<AssetServer>,
    input: Res<ButtonInput<MouseButton>>,
    mut inventory: Single<&mut Inventory>,
    pointer: Single<&PointerInteraction, With<PlayerCursor>>,
    locations: Query<&Transform, With<ItemsPlaceable>>,
) {
    if !input.just_pressed(MouseButton::Right) {
        return;
    }

    if let Some((
        entity,
        HitData {
            position: Some(position),
            normal: Some(normal),
            ..
        },
    )) = pointer.get_nearest_hit()
    {
        if locations.get(*entity).is_err() {
            return;
        }

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
}

fn discard_item(
    input: Res<ButtonInput<MouseButton>>,
    mut inventory: Single<&mut Inventory>,
    pointer: Single<&PointerInteraction, With<PlayerCursor>>,
    locations: Query<&Transform, With<ItemsDiscardable>>,
) {
    if !input.just_pressed(MouseButton::Right) {
        return;
    }

    if let Some((entity, _hit)) = pointer.get_nearest_hit() {
        if locations.get(*entity).is_err() {
            return;
        }

        inventory.item = None;
    }
}
