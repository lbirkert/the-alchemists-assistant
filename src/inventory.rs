use bevy::picking::backend::HitData;

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(GameRunning, pickup_item);
    app.add_systems(GameRunning, place_item);
}

const ITEM_SIZE: f32 = 0.3;

#[derive(Component, Default, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct ItemsPlaceable;

#[derive(Reflect, Debug, Clone, Copy)]
// TODO: needs better names?
pub enum HerbKind {
    Red,
    Orange,
    Yellow,
    Blue,
    Purple,
    Pink,
    White,
    Green,
}

impl HerbKind {
    pub fn display_name(&self) -> &'static str {
        match self {
            HerbKind::Red => "Red Herb",
            HerbKind::Orange => "Orange Herb",
            HerbKind::Yellow => "Yellow Herb",
            HerbKind::Blue => "Blue Herb",
            HerbKind::Purple => "Purple Herb",
            HerbKind::Pink => "Pink Herb",
            HerbKind::White => "White Herb",
            HerbKind::Green => "Green Herb",
        }
    }

    fn scene(self) -> impl Scene {
        bsn! {
            Mesh3d(asset_value(Cuboid::new(ITEM_SIZE, ITEM_SIZE, ITEM_SIZE)))
            MeshMaterial3d::<StandardMaterial>(asset_value(Color::BLACK))
        }
    }
}

// we might want to use https://github.com/FraserLee/bevy_sprite3d/blob/main/examples/sprite.rs
// for rendering the items. -> Ask faervan

#[derive(Component, Reflect, Debug, Clone, Copy)]
#[reflect(Component)]
pub enum Item {
    FishSalmon,

    Herb {
        kind: HerbKind,
        // martered: bool, // gemörsert, TODO: brauchen nen shader/textures dafür
    },
}

impl Item {
    pub fn display_name(&self) -> &'static str {
        match self {
            Item::Herb { kind } => kind.display_name(),
            _ => "Unknown",
        }
    }

    pub fn scene(self) -> impl Scene {
        let scene = match self {
            Item::Herb { kind } => kind.scene(),
            _ => todo!(),
        };

        bsn! {
            @scene
            self
        }
    }

    fn physics_scene(self, pos: Vec3, vel: Vec3) -> impl Scene {
        let scene = self.scene();
        bsn! {
            RigidBody::Dynamic
            LinearVelocity(vel)
            Transform::from_translation(pos)
            Collider::sphere(ITEM_SIZE/2.0)
            Pickable
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
    input: Res<ButtonInput<MouseButton>>,
    mut inventory: Single<&mut Inventory>,
    pointer: Single<&PointerInteraction, With<PlayerCursor>>,
    items: Query<(&Item, &Transform)>,
) {
    if !input.just_pressed(MouseButton::Right) {
        return;
    }

    if let Some((entity, _hit)) = pointer.get_nearest_hit()
        && let Ok((item, item_transform)) = items.get(*entity)
    {
        let old_item = inventory.item.replace(*item);
        if let Some(old_item) = old_item {
            let item_scene = old_item.physics_scene(item_transform.translation, Vec3::ZERO);
            commands.spawn_scene(item_scene);
        }

        // destroy old item
        commands.entity(*entity).despawn();
    }
}

fn place_item(
    mut commands: Commands,
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

        let item_scene = item.physics_scene(position + normal * ITEM_SIZE / 2.0, Vec3::ZERO);
        commands.spawn_scene(item_scene);
    }
}
