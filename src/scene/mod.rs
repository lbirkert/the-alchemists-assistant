use crate::prelude::*;

mod stations;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(ON_GAME_ENTER, spawn_scene);
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[require(
    RigidBody::Static,
    ItemsPlaceable,
    CollisionLayers::new(GameLayer::Environment, GameLayer::all_bits())
)]
pub struct EnvironmentObject;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[require(
    ColliderConstructor::TrimeshFromMeshWithConfig(TrimeshFlags::FIX_INTERNAL_EDGES),
    EnvironmentObject
)]
struct ColliderFromMesh;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[require(
    EnvironmentObject,
    PointLight {
        intensity: 600000.0,
        ..Default::default()
    },
)]
struct GameLight;

fn spawn_scene(mut commands: Commands) {
    commands.spawn_scene(bsn! {
        #GameScene
        WorldAssetRoot("TheAlchemistsAssistant.glb#Scene0")
    });
}
