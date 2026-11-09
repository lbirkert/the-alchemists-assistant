use crate::prelude::*;

use bevy_easings::EasingComponent;
use bevy_flora::{FloraPlugin, Growing, ProceduralTree, TreeAge, TreePreset};

mod calendar;
mod score_board;
pub mod stations;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((stations::plugin, calendar::plugin, score_board::plugin));
    app.add_plugins(FloraPlugin);

    app.add_systems(GameUpdate, spawn_scene.run_if(state_changed::<GameScene>));
    app.add_systems(GameUpdate, rotate_clock_hand);

    app.insert_resource(GlobalAmbientLight {
        color: Color::srgba(1.0, 1.0, 0.8, 1.0),
        brightness: 100.0,
        ..default()
    });
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[require(
    RigidBody::Static,
    CollisionLayers::new(GameLayer::Environment, GameLayer::all_bits())
)]
/// TODO! Unused?
pub struct EnvironmentObject;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[require(
    ColliderConstructor::TrimeshFromMeshWithConfig(TrimeshFlags::FIX_INTERNAL_EDGES),
    EnvironmentObject
)]
#[type_path = "the_alchemist_assistant::scene"]
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
#[type_path = "the_alchemist_assistant::scene"]
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
#[type_path = "the_alchemist_assistant::scene"]
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
#[type_path = "the_alchemist_assistant::scene"]
struct Tree;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[require(Interactable::Bell)]
#[type_path = "the_alchemist_assistant::scene"]
pub struct NpcBell;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[type_path = "the_alchemist_assistant::scene"]
pub struct NpcStickyNotes;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[require(Interactable::NpcTable)]
#[type_path = "the_alchemist_assistant::scene"]
pub struct NpcTable {
    pub potions: Vec<(Entity, PotionData)>,
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[type_path = "the_alchemist_assistant::scene"]
struct GameClockHand;

impl NpcTable {
    pub fn validatable_potion(&mut self) -> Option<(Entity, PotionData)> {
        let idx = self.potions.iter().position(|(_, p)| p.can_validate())?;
        Some(self.potions.remove(idx))
    }
}

impl GameScene {
    pub fn asset_path(&self) -> &'static str {
        match self {
            Self::Game => "TheAlchemistsAssistant.glb#Scene0",
            Self::Intro => "Intro-Office.glb#Scene0",
        }
    }
}

fn spawn_scene(mut commands: Commands, scene: Res<State<GameScene>>) {
    let scene_path = scene.asset_path();
    let scene = *scene.get();
    commands.spawn_scene(bsn! {
        #GameScene
        WorldAssetRoot(scene_path)
        DespawnOnExit<GameScene>(scene)
    });
}

fn rotate_clock_hand(
    manager: Res<GameManager>,
    mut commands: Commands,
    query: Query<(Entity, &Transform), (With<GameClockHand>, Without<EasingComponent<Transform>>)>,
) {
    let rotation = Quat::from_rotation_z(-TAU * manager.game_hour() as f32 / 24.);
    for (entity, transform) in query {
        if rotation.angle_between(transform.rotation) > 0.1 {
            commands.entity(entity).insert(transform.ease_to(
                transform.with_rotation(rotation),
                EaseFunction::ExponentialIn,
                EasingType::Once {
                    duration: Duration::from_millis(350),
                },
            ));
        }
    }
}
