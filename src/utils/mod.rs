use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(GameUpdate, remove_meshes);
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

pub fn find<C: Component>(
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
