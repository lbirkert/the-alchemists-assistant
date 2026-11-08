use crate::prelude::*;
use std::hash::{DefaultHasher, Hash, Hasher};

pub mod color_overlay;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(color_overlay::plugin);

    app.add_systems(GameUpdate, (remove_meshes, billboard));
}

#[derive(Component, Reflect)]
#[reflect(Component)]
#[require(Visibility::Hidden)]
/// This component is added to meshes that only exist to tell avian the shape of a collider (see
/// [`ColliderConstructor`]). After avian added the collider, we can remove the mesh and this
/// marker.
struct ColliderConstructerRemoveMesh;

fn remove_meshes(
    mut commands: Commands,
    query: Query<Entity, (With<ColliderConstructerRemoveMesh>, With<Collider>)>,
) {
    for entity in query {
        commands.entity(entity).remove::<(
            ColliderConstructerRemoveMesh,
            Mesh3d,
            MeshMaterial3d<StandardMaterial>,
        )>();
    }
}

pub fn find_parent<C: Component>(
    mut entity: Entity,
    query: &Query<(Option<&ChildOf>, Has<C>)>,
) -> Option<Entity> {
    loop {
        let (parent, has_item) = query.get(entity).ok()?;

        if has_item {
            return Some(entity);
        }

        entity = parent?.0;
    }
}

pub fn items_to_color(items: &Vec<Item>) -> Color {
    let mut hasher = DefaultHasher::new();
    items.hash(&mut hasher);

    let hash = hasher.finish();

    let hue = ((hash / 256) % 256) as f32;

    Color::hsv(hue, 1.0, 1.0)
}

#[derive(Component, Reflect, Default, Clone)]
#[reflect(Component)]
pub struct Billboarded;

fn billboard(
    query: Query<&mut Transform, (With<Billboarded>, Without<Camera3d>)>,
    camera: Single<&Transform, With<Camera3d>>,
) {
    for mut transform in query {
        transform.rotation = Quat::from_rotation_arc(
            Vec3::NEG_Z,
            camera.forward().as_vec3().with_y(0.).normalize_or_zero(),
        );
    }
}
