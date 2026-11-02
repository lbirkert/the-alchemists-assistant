use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(GameRunning, pickup_item);
    app.add_systems(GameRunning, place_item);
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

        let asset =
            asset_server.load(GltfAssetLabel::Scene(0).from_asset(format!("models/{}", asset)));

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
    hits_query: Query<&ShapeHits>,
    items: Query<(&Item, &Transform)>,
) {
    if !input.just_pressed(MouseButton::Right) {
        return;
    }

    for hits in hits_query {
        for hit in hits.iter_sorted() {
            let Ok((item, item_transform)) = items.get(hit.entity) else {
                continue;
            };

            let old_item = std::mem::replace(&mut inventory.item, Some(item.clone()));
            if let Some(old_item) = old_item {
                let item_scene =
                    old_item.physics_scene(&assets_server, item_transform.translation, Vec3::ZERO);
                commands.spawn_scene(item_scene);
            }

            // destroy old item
            commands.entity(hit.entity).despawn();
        }
    }
}

fn place_item(
    mut commands: Commands,
    assets_server: Res<AssetServer>,
    input: Res<ButtonInput<MouseButton>>,
    mut inventory: Single<&mut Inventory>,
    hits_query: Query<&ShapeHits>,
    locations: Query<&Transform, With<ItemsPlaceable>>,
) {
    if !input.just_pressed(MouseButton::Right) {
        return;
    }

    for hits in hits_query {
        for hit in hits.iter_sorted() {
            if locations.get(hit.entity).is_err() {
                continue;
            }

            let Some(item) = inventory.item.take() else {
                return;
            };

            let item_scene = item.physics_scene(
                &assets_server,
                hit.point1 + hit.normal1 * ITEM_SIZE / 2.0,
                Vec3::ZERO,
            );
            commands.spawn_scene(item_scene);

            return;
        }
    }
}
