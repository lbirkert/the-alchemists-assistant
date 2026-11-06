use crate::prelude::*;

use bevy_flora::{FloraPlugin, Growing, ProceduralTree, TreeAge, TreePreset};

pub mod stations;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins(stations::plugin);
    app.add_plugins(FloraPlugin);

    app.add_systems(ON_GAME_ENTER, spawn_scene);
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[require(
    RigidBody::Static,
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
    Name::new("Sun Light"),
    PointLight {
        intensity: 400000.0,
        range: 400.0,
        radius: 400.0,
        color: Color::srgba(1.0, 1.0, 0.8, 1.0),
        shadow_maps_enabled: true,
        shadow_map_near_z: 0.4,
        contact_shadows_enabled: false,
        ..Default::default()
    },
)]
struct SunLight;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[require(
    Name::new("Cauldron Light"),
    PointLight {
        intensity: 80000.0,
        range: 5.0,
        radius: 5.0,
        color: Color::srgba(0.0, 1.0, 0.0, 1.0),
        shadow_maps_enabled: false,
        ..Default::default()
    },
)]
struct CauldronLight;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[require(
    Name::new("Tree"),
    ProceduralTree(TreePreset::Oak.params()),
    TreeAge(10.0),
    Growing {
        rate: 0.0,
    },
)]
struct Tree;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[require(Interactable::Bell)]
pub struct NpcBell;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct NpcStickyNotes;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct NpcTable {
    pub potions: Vec<(Entity, Potion)>,
}

impl NpcTable {
    pub fn validatable_potion(&mut self) -> Option<(Entity, Potion)> {
        let idx = self.potions.iter().position(|(_, p)| p.can_validate())?;
        Some(self.potions.remove(idx))
    }
}

fn spawn_scene(mut commands: Commands) {
    commands.spawn_scene(bsn! {
        #GameScene
        WorldAssetRoot("TheAlchemistsAssistant.glb#Scene0")
        DespawnOnExit<GameState>(GameState::Game)
    });
    commands.spawn_scene(bsn! {
        #Alchemist
        WorldAssetRoot("Alchemist.glb#Scene0")
        Transform::from_xyz(1., 0., 2.)
        DespawnOnExit<GameState>(GameState::Game)
    });

    commands.insert_resource(GlobalAmbientLight {
        color: Color::srgba(1.0, 1.0, 0.8, 1.0),
        brightness: 100.0,
        ..default()
    });
}
