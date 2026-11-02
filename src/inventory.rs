use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(GameRunning, pickup_item);
    app.add_systems(GameRunning, place_item);
}

#[derive(Component, Reflect, Debug, Clone)]
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
            Mesh3d(asset_value(Cuboid::new(0.5, 0.5, 0.5)))
            MeshMaterial3d::<StandardMaterial>(asset_value(Color::BLACK))
            Collider::sphere(0.5)
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

    fn scene(self, pos: Vec3, vel: Vec3) -> impl Scene {
        let scene = match self {
            Item::Herb { kind } => kind.scene(),
            _ => todo!(),
        };

        bsn! {
            RigidBody::Dynamic
            LinearVelocity(vel)
            Transform::from_translation(pos)
            @scene
            self
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
    mut query: Query<(&mut Inventory, &RayHits)>,
    items: Query<(&Item, &Transform)>,
) {
    let (mut inventory, hits) = query.single_mut().unwrap();

    if !input.just_pressed(MouseButton::Right) {
        return;
    }

    for hit in hits.iter_sorted() {
        let Ok((item, item_transform)) = items.get(hit.entity) else {
            continue;
        };

        let old_item = std::mem::replace(&mut inventory.item, Some(item.clone()));
        if let Some(old_item) = old_item {
            let item_scene = old_item.scene(item_transform.translation, Vec3::ZERO);
            commands.spawn_scene(item_scene);
        }

        // destroy old item
        commands.entity(hit.entity).despawn();
    }
}

fn place_item(
    mut commands: Commands,
    input: Res<ButtonInput<MouseButton>>,
    mut query: Query<(&mut Inventory, &RayHits)>,
    locations: Query<&Transform, With<ItemsPlaceable>>,
) {
    let (mut inventory, hits) = query.single_mut().unwrap();

    if !input.just_pressed(MouseButton::Right) {
        return;
    }

    let Some(item) = inventory.item.take() else {
        return;
    };

    for hit in hits.iter_sorted() {
        let Ok(transform) = locations.get(hit.entity) else {
            continue;
        };

        let item_scene = item.scene(transform.translation, Vec3::ZERO);
        commands.spawn_scene(item_scene);

        return;
    }
}
