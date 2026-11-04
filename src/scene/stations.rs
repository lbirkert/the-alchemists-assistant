use bevy::world_serialization::WorldInstanceReady;

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(GameUpdate, (mortar_interaction, update_mortar));
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[require(EnvironmentObject, Collider::cuboid(2., 0.5, 0.8))]
struct AlchemyTable;

#[derive(Component, Reflect, Default)]
#[reflect(Component, Default)]
#[require(Interactable::Mortar)]
#[component(on_add)]
pub struct MortarStation {
    pub state: MortarState,
    set_animation: bool,
}

#[derive(Reflect, Default)]
pub enum MortarState {
    #[default]
    Idle,
    Processing {
        working: bool,
        item: Item,
        item_entity: Entity,
        timer: Timer,
    },
    Done {
        item: Item,
        item_entity: Entity,
    },
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
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut inventory: Single<&mut Inventory>,
    mut messages: MessageReader<PlayerInteraction>,
    mut mortars: Query<&mut MortarStation>,
) {
    if let Some(PlayerInteraction {
        entity,
        kind: Interactable::Mortar,
        ..
    }) = messages.read().next()
        && let Ok(mut mortar) = mortars.get_mut(*entity)
    {
        match (&mortar.state, inventory.item) {
            (MortarState::Idle, Some(item)) => {
                let scene = item.scene(&asset_server, Vec3::new(0., 0.2, 0.));
                let item_entity = commands
                    .spawn_scene(bsn! {
                        #MortarItem
                        ChildOf({*entity})
                        @scene
                    })
                    .id();
                mortar.state = MortarState::Processing {
                    working: true,
                    item,
                    item_entity,
                    timer: Timer::new(Duration::from_millis(2000), TimerMode::Once),
                };
                mortar.set_animation = true;
            }
            (MortarState::Processing { .. }, _) => {
                mortar.set_animation = true;
            }
            (MortarState::Done { item, item_entity }, None) => {
                commands.entity(*item_entity).despawn();
                inventory.item = Some(*item);
                mortar.state = MortarState::Idle;
                mortar.set_animation = true;
            }
            _ => {}
        }
    }
}

fn update_mortar(
    focus: Res<PlayerFocus>,
    input: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    mortars: Query<(Entity, &ArmatureHandle, &mut MortarStation)>,
    mut mortar_armatures: Query<(
        &mut AnimationPlayer,
        &mut AnimationTransitions,
        &MortarAnimations,
    )>,
) {
    for (entity, ArmatureHandle(armature_entity), mut mortar) in mortars {
        if mortar.set_animation
            && let Ok((mut animation_player, mut transitions, animations)) =
                mortar_armatures.get_mut(*armature_entity)
        {
            mortar.set_animation = false;
            let animation = match mortar.state {
                MortarState::Processing { working: true, .. } => animations.pestle,
                _ => animations.idle,
            };
            transitions
                .play(&mut animation_player, animation, Duration::from_millis(100))
                .repeat();
        }
        match &mut mortar.state {
            MortarState::Idle => {}
            MortarState::Processing {
                working,
                item,
                item_entity,
                timer,
            } if *working => {
                if input.pressed(MouseButton::Right)
                    && focus.interactable == Some((entity, Interactable::Mortar))
                {
                    timer.tick(time.delta());
                    if !timer.just_finished() {
                        continue;
                    }
                }
                *working = false;
                mortar.set_animation = true;
            }
            MortarState::Processing { .. } => {}
            MortarState::Done { .. } => {}
        }
    }
}
