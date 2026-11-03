use bevy::picking::backend::HitData;

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(GameUpdate, pickup_item);
    app.add_systems(GameUpdate, place_item);
}

const ITEM_SIZE: f32 = 0.3;

#[derive(Component, Default, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct ItemsPlaceable;

// we might want to use https://github.com/FraserLee/bevy_sprite3d/blob/main/examples/sprite.rs
// for rendering the items. -> Ask faervan

#[derive(Component, Reflect, Debug, Clone, Copy)]
#[reflect(Component)]
pub enum Item {
    Garlic,
    Crystal,
    Grass,
    Flowers,
}

impl Item {
    pub fn display_name(&self) -> &'static str {
        match self {
            _ => "Unknown",
        }
    }

    pub fn scene(self, asset_server: &AssetServer, pos: Vec3) -> impl Scene {
        let asset = match self {
            Item::Garlic => "Garlic.glb",
            Item::Grass => "Grass.glb",
            Item::Flowers => "Flowers.glb",
            Item::Crystal => "Crystal.glb",
            _ => todo!(),
        };

        let asset = asset_server.load(GltfAssetLabel::Scene(0).from_asset(asset));

        bsn! {
            self
            WorldAssetRoot(asset)
            Transform::from_translation(pos).with_scale(Vec3::splat(ITEM_SIZE))
        }
    }

    fn physics_scene(self, asset_server: &AssetServer, pos: Vec3, vel: Vec3) -> impl Scene {
        let scene = self.scene(asset_server, pos);
        bsn! {
            RigidBody::Dynamic
            LinearVelocity(vel)
            Collider::cuboid(1.0, 1.0, 1.0)
            @scene
        }
    }
}

#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect(Component)]
pub struct Inventory {
    pub item: Option<Item>,
    // TODO: item_lh and item_rh
}

fn pickup_item(
    mut commands: Commands,
    assets_server: Res<AssetServer>,
    input: Res<ButtonInput<MouseButton>>,
    mut inventory: Single<&mut Inventory>,
    pointer: Single<&PointerInteraction, With<PlayerCursor>>,
    item_query: Query<(Option<&ChildOf>, Option<&Item>, &Transform)>,
) {
    if !input.just_pressed(MouseButton::Right) {
        return;
    }

    if let Some((entity, _hit)) = pointer.get_nearest_hit()
        && let Some((item, item_transform)) = find_item(*entity, &item_query)
    {
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

fn find_item<'a>(
    entity: Entity,
    query: &'a Query<(Option<&ChildOf>, Option<&Item>, &Transform)>,
) -> Option<(&'a Item, &'a Transform)> {
    let (parent, item, transform) = query.get(entity).ok()?;
    if let Some(item) = item {
        return Some((item, transform));
    }
    let parent = parent?;
    find_item(parent.0, query)
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
