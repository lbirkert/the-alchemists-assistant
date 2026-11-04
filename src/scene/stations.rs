use bevy::world_serialization::WorldInstanceReady;

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(GameUpdate, (mortar_interaction, update_mortar));
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[require(EnvironmentObject, Collider::cuboid(2., 0.5, 0.8))]
struct AlchemyTable;

#[derive(Component, Reflect)]
#[reflect(Component, Default)]
#[require(Interactable::Mortar)]
#[component(on_add)]
pub struct MortarStation {
    pub working: bool,
    timer: Timer,
}

impl Default for MortarStation {
    fn default() -> Self {
        let mut timer = Timer::new(Duration::from_millis(2000), TimerMode::Once);
        timer.almost_finish();
        Self {
            working: false,
            timer,
        }
    }
}

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
#[component(on_add)]
struct MortarStationArmature;

#[derive(Component, Reflect)]
#[reflect(Component)]
struct ArmatureHandle(Entity);

#[derive(Component, Reflect)]
#[reflect(Component)]
struct MortarAnimations {
    idle: AnimationNodeIndex,
    pestle: AnimationNodeIndex,
}

impl MortarStation {
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        world.commands().entity(ctx.entity).apply_scene(bsn! {
            WorldAssetRoot("Mortar.glb#Scene0")
            on(|
                event: On<WorldInstanceReady>,
                mut commands: Commands,
                armatures: Query<Entity, With<MortarStationArmature>>,
                mortars: Query<(Option<&ChildOf>, Has<MortarStation>)>
            | {
                for armature_entity in armatures {
                    if let Some(mortar_entity)
                        = crate::utils::find_parent(armature_entity, &mortars)
                            && mortar_entity == event.entity {
                        commands.entity(event.entity).insert(ArmatureHandle(armature_entity));
                        return;
                    }
                }
                error!("Failed to find mortar armature entity for mortar {}", event.entity);
            })
        });
    }
}

impl MortarStationArmature {
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        let asset_server = world.resource::<AssetServer>();
        let (graph, animations) = AnimationGraph::from_clips(
            ["Mortar.glb#Animation0", "Mortar.glb#Animation1"].map(|s| asset_server.load(s)),
        );
        let mut graphs = world.resource_mut::<Assets<AnimationGraph>>();
        let graph_handle = graphs.add(graph);

        let Some([idle, pestle]) = animations.as_array().copied() else {
            error!("Failed to load mortar animations");
            return;
        };

        world.commands().entity(ctx.entity).insert((
            AnimationGraphHandle(graph_handle),
            AnimationTransitions::new(),
            MortarAnimations { idle, pestle },
        ));
    }
}

fn mortar_interaction(
    mut messages: MessageReader<PlayerInteraction>,
    mut mortars: Query<(Entity, &mut MortarStation)>,
) {
    if let Some(PlayerInteraction {
        entity,
        kind: Interactable::Mortar,
        ..
    }) = messages.read().next()
        && let Ok((_, mut mortar)) = mortars.get_mut(*entity)
    {
        mortar.working = true;
    }
}

fn update_mortar(
    time: Res<Time>,
    mortars: Query<(&ArmatureHandle, &mut MortarStation)>,
    mut mortar_armatures: Query<(
        &mut AnimationPlayer,
        &mut AnimationTransitions,
        &MortarAnimations,
    )>,
) {
    for (ArmatureHandle(entity), mut mortar) in mortars {
        mortar.timer.tick(time.delta());
        if mortar.timer.just_finished() {
            mortar.working = false;
            if let Ok((mut animation_player, mut transitions, animations)) =
                mortar_armatures.get_mut(*entity)
            {
                transitions.play(
                    &mut animation_player,
                    animations.idle,
                    Duration::from_millis(100),
                );
            }
        } else if mortar.timer.is_finished()
            && mortar.working
            && let Ok((mut animation_player, mut transitions, armature)) =
                mortar_armatures.get_mut(*entity)
        {
            mortar.timer.reset();
            transitions
                .play(
                    &mut animation_player,
                    armature.pestle,
                    Duration::from_millis(100),
                )
                .repeat();
        }
    }
}
