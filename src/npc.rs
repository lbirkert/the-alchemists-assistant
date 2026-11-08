// the npc is a state machine that reacts to game state changes
//
// Idle: on new day -> GreetPlayer
// Idle: on bell pressed -> WalkToTable
// WalkToTable: on table has potion -> TakePotion(..)
// TakePotion: on walked back -> Idle
// WalkToTable: on table has no potion -> WalkBack + say something funny
// Idle: on # recipes < MAX_RECIPES -> BringRecipe
// Idle: on time > DAY_DURATION -> SayByeToPlayer
// SayByeToPlayer: on finished -> Idle + set day_ended to true

use bevy::{picking::backend::HitData, world_serialization::WorldInstanceReady};
use bevy_easings::{EaseMethod, EasingComponent};

use crate::{
    manager::DayState,
    potion::PotionReport,
    prelude::*,
    recipe::GREETINGS,
    scene::{NpcStickyNotes, NpcTable},
    sounds::movement::IsMoving,
    utils::Billboarded,
};

pub fn plugin(app: &mut App) {
    app.add_systems(
        GameUpdate,
        (
            update_walking_npc,
            update_idle_npc,
            update_speaking_npc,
            listen_for_bell,
            update_npc_animation,
            update_npc_speech_text,
        ),
    );
}

#[derive(Component, Reflect, Clone, Default)]
#[reflect(Component)]
#[component(on_add)]
#[type_path = "the_alchemist_assistant::npc"]
pub struct Npc {
    state_start: f32, // state starting time
    pub state: NpcState,
}

impl Npc {
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        let speech = npc_speech_scene(&mut world);
        world.commands().entity(ctx.entity).apply_scene(bsn! {
            WorldAssetRoot("Alchemist.glb#Scene0")
            on(instance_ready_setup_armature)
            on(instance_ready_find_target_positions)
            // We can't insert the collider directly, else avians ColliderFromMesh will break.
            on(|event: On<WorldInstanceReady>, mut commands: Commands| {
                commands.spawn((
                    ChildOf(event.entity),
                    Transform::from_xyz(0., 1., 0.),
                    RigidBody::Kinematic,
                    Collider::capsule(0.5, 2.),
                ));
            })
            @speech
        });
    }
}

#[derive(Reflect, Clone, Default, Debug)]
pub enum NpcState {
    #[default]
    Idle,
    PreparingToSpeak {
        timer: Timer,
        message: MessageToPlayer,
    },
    SpeakingToPlayer(MessageToPlayer),
    WalkIntoRoom(WalkIntoRoomReason),
    WalkOutOfRoom(WalkOutOfRoomReason),
}

#[derive(Reflect, Clone, Debug)]
pub struct MessageToPlayer {
    message: String,
    context: MessageContext,
}

#[derive(Reflect, Clone, Debug)]
enum MessageContext {
    None,
    StartDay,
    EndDay,
}

#[derive(Reflect, Clone, Debug)]
pub enum WalkIntoRoomReason {
    BellPressed,
    BringRecipes(Vec<Recipe>),
    SayToPlayer(MessageToPlayer),
}

#[derive(Reflect, Clone, Debug)]
pub enum WalkOutOfRoomReason {
    SayToPlayer(MessageToPlayer),
    TakePotion(PotionData),
    NoPotion,
}

impl NpcState {
    fn say_to_player(message: impl ToString, context: MessageContext) -> Self {
        Self::WalkIntoRoom(WalkIntoRoomReason::SayToPlayer(MessageToPlayer {
            message: message.to_string(),
            context,
        }))
    }
}

fn update_idle_npc(
    upgrades: Res<GameUpgrades>,
    mut manager: Single<&mut GameManager>,
    mut npc: Single<&mut Npc>,
    time: Res<Time>,
) {
    if !matches!(npc.state, NpcState::Idle) {
        return;
    }

    match manager.day_state {
        DayState::Started => {
            let mut rng = rand::rng();
            npc.state = NpcState::say_to_player(
                GREETINGS.choose(&mut rng).unwrap(),
                MessageContext::StartDay,
            );
            manager.day_state = DayState::Running;
        }
        DayState::Running => {
            if manager.should_end_day() {
                info!("It is {} o´clock, ending day", manager.game_hour());
                npc.state_start = time.elapsed_secs();
                npc.state = NpcState::say_to_player(
                    "It is evening already.\n\
                        I will go to sleep now.\n\
                        You should too.",
                    MessageContext::EndDay,
                );
                // TODO: proper messages

                return;
            }

            if manager.should_bring_recipe() {
                // collect all recipes at once so you don't have to walk so much
                let mut recipes = Vec::new();
                while manager.should_bring_recipe() {
                    let recipe = Recipe::generate(&upgrades);
                    manager.recipes.push(recipe.clone());
                    recipes.push(recipe);
                }

                npc.state_start = time.elapsed_secs();
                npc.state = NpcState::WalkIntoRoom(WalkIntoRoomReason::BringRecipes(recipes));
            }
        }
        DayState::Ended => {
            if manager.should_start_day() {
                info!("It is {} o´clock, starting new day", manager.game_hour());
                manager.day_state = DayState::Started;
            }
        }
    }
}

fn listen_for_bell(
    mut messages: MessageReader<PlayerInteraction>,
    mut npc: Single<&mut Npc>,
    manager: Single<&GameManager>,
    time: Res<Time>,
) {
    if manager.day_state != DayState::Running {
        return;
    }

    let Some(PlayerInteraction {
        kind: Interactable::Bell,
        hit: HitData { .. },
        ..
    }) = messages.read().next()
    else {
        return;
    };

    if !matches!(npc.state, NpcState::Idle) {
        return;
    }

    npc.state_start = time.elapsed_secs();
    npc.state = NpcState::WalkIntoRoom(WalkIntoRoomReason::BellPressed);
}

fn update_walking_npc(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut manager: Single<&mut GameManager>,
    mut npc_query: Single<(&mut Transform, &mut Npc, &NpcTargetPositions), Without<NpcArmature>>,
    npc_armature: Single<(Entity, &Transform, Has<EasingComponent<Transform>>), With<NpcArmature>>,
    mut npc_table: Single<&mut NpcTable>,
    npc_sticky_notes: Single<Entity, With<NpcStickyNotes>>,
    time: Res<Time>,
) {
    if manager.day_state == DayState::Ended {
        return;
    }

    let (ref mut transform, ref mut npc, target_positions) = *npc_query;

    let target_pos = match npc.state {
        NpcState::WalkIntoRoom(WalkIntoRoomReason::SayToPlayer(..)) => {
            target_positions.inside_speak
        }
        NpcState::WalkIntoRoom(..) => target_positions.inside_interact,
        NpcState::WalkOutOfRoom(..) => target_positions.outside,
        _ => return,
    };

    let (armature_entity, armature_transform, has_easing) = *npc_armature;
    if !has_easing {
        rotate_if_needed(
            commands.entity(armature_entity),
            target_pos - transform.translation,
            armature_transform,
        );
    }

    let has_reached_target = walk_npc(transform, &time, target_pos);
    if !has_reached_target {
        return;
    }

    npc.state_start = time.elapsed_secs();
    npc.state = match npc.state {
        NpcState::WalkIntoRoom(ref reason) => match reason {
            WalkIntoRoomReason::BellPressed => match npc_table.validatable_potion() {
                Some((entity, potion)) => {
                    commands.entity(entity).despawn();
                    NpcState::WalkOutOfRoom(WalkOutOfRoomReason::TakePotion(potion))
                }
                None => NpcState::WalkOutOfRoom(WalkOutOfRoomReason::NoPotion),
            },
            WalkIntoRoomReason::BringRecipes(recipes) => {
                let mut offset = 0.0; // TODO: also account for existing sticky notes
                for recipe in recipes.iter() {
                    // spawn the sticky note
                    let scene = StickyNote {
                        text: recipe.task_description.clone(),
                        recipe: Some(recipe.clone()),
                    }
                    .physics_scene(
                        &asset_server,
                        Transform::from_translation(Vec3::new(0.0, offset, 0.0))
                            .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
                    );
                    commands
                        .spawn_scene(scene)
                        .insert(ChildOf(*npc_sticky_notes));

                    offset += 0.02;
                }

                // check table for potions
                match npc_table.validatable_potion() {
                    Some((entity, potion)) => {
                        commands.entity(entity).despawn();
                        NpcState::WalkOutOfRoom(WalkOutOfRoomReason::TakePotion(potion))
                    }
                    None => NpcState::WalkOutOfRoom(WalkOutOfRoomReason::NoPotion),
                }
            }
            WalkIntoRoomReason::SayToPlayer(msg) => NpcState::PreparingToSpeak {
                timer: Timer::new(Duration::from_secs(2), TimerMode::Once),
                message: msg.clone(),
            },
        },
        NpcState::WalkOutOfRoom(ref reason) => match reason {
            WalkOutOfRoomReason::NoPotion => NpcState::Idle,
            WalkOutOfRoomReason::TakePotion(potion) => {
                let Some(recipe) = potion.recipe.as_ref() else {
                    return;
                };

                let report = potion.validate().unwrap();
                manager.score_potion(report.is_valid());

                let idx = manager.recipes.iter().position(|r| r == recipe);
                manager.recipes.remove(idx.unwrap());

                debug!("PotionReport: {report:#?}");

                match report {
                    PotionReport::Valid => NpcState::Idle,
                    PotionReport::WrongOrder => NpcState::say_to_player(
                        "Your potion seems correct but on closer inspection, something is off.",
                        MessageContext::None,
                    ),
                    PotionReport::Invalid { .. } => {
                        NpcState::say_to_player("Invalid potion", MessageContext::None)
                    }
                }
            }
            WalkOutOfRoomReason::SayToPlayer(msg) => {
                if matches!(msg.context, MessageContext::EndDay) {
                    manager.day_state = DayState::Ended;
                }

                NpcState::Idle
            }
        },
        _ => panic!(),
    };
}

fn update_speaking_npc(manager: Single<&GameManager>, mut npc: Single<&mut Npc>, time: Res<Time>) {
    if manager.day_state == DayState::Ended {
        return;
    }

    let NpcState::SpeakingToPlayer(ref msg) = npc.state else {
        if let NpcState::PreparingToSpeak { timer, message } = &mut npc.state {
            timer.tick(time.delta());
            if timer.is_finished() {
                npc.state = NpcState::SpeakingToPlayer(message.clone());
            }
        }
        return;
    };

    // Set speaking duration based on amount of spoken words.
    let mut duration = 3. + msg.message.split_whitespace().count() as f32 * 0.3;
    if let MessageContext::EndDay = msg.context {
        // Show bye messages always longer.
        duration *= 2.;
    }
    duration = duration.min(20.);

    let now = time.elapsed_secs();
    if now - npc.state_start <= duration {
        return;
    }

    npc.state = NpcState::WalkOutOfRoom(WalkOutOfRoomReason::SayToPlayer(msg.clone()));
    npc.state_start = now;
}

fn rotate_if_needed(mut commands: EntityCommands, direction: Vec3, transform: &Transform) {
    let forward = transform.back().as_vec3();
    if direction.angle_between(forward) > 0.1 && direction.length_squared() > 1. {
        let rotation = Quat::from_rotation_arc(Vec3::NEG_Z, forward);
        commands.insert(transform.ease_to(
            transform.with_rotation(rotation),
            EaseMethod::Linear,
            EasingType::Once {
                duration: Duration::from_millis(300),
            },
        ));
    }
}

fn walk_npc(transform: &mut Transform, time: &Time, target_pos: Vec3) -> bool {
    const NPC_SPEED: f32 = 2.0;

    let delta = target_pos - transform.translation;
    if delta.length_squared() < 1e-3 {
        return true;
    }

    let dt = time.delta().as_secs_f32();
    let movement = delta.normalize() * dt * NPC_SPEED;
    transform.translation += movement;

    false
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[type_path = "the_alchemist_assistant::npc"]
struct NpcPosOutside;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[type_path = "the_alchemist_assistant::npc"]
struct NpcPosInsideSpeakToPlayer;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[type_path = "the_alchemist_assistant::npc"]
struct NpcPosInsideInteractWithTable;

#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect(Component)]
struct NpcTargetPositions {
    outside: Vec3,
    inside_speak: Vec3,
    inside_interact: Vec3,
}

fn instance_ready_find_target_positions(
    event: On<WorldInstanceReady>,
    mut commands: Commands,
    npc_pos_outside: Option<Single<(Entity, &Transform), With<NpcPosOutside>>>,
    npc_pos_speak: Option<Single<(Entity, &Transform), With<NpcPosInsideSpeakToPlayer>>>,
    npc_pos_interact: Option<Single<(Entity, &Transform), With<NpcPosInsideInteractWithTable>>>,
) {
    let Some((e1, outside)) = npc_pos_outside.as_deref() else {
        error!("Failed to find NpcPosOutside");
        return;
    };
    let Some((e2, speak)) = npc_pos_speak.as_deref() else {
        error!("Failed to find NpcPosInsideSpeakToPlayer");
        return;
    };
    let Some((e3, interact)) = npc_pos_interact.as_deref() else {
        error!("Failed to find NpcPosInsideInteractWithTable");
        return;
    };

    commands.entity(*e1).despawn();
    commands.entity(*e2).despawn();
    commands.entity(*e3).despawn();

    commands.entity(event.entity).insert(NpcTargetPositions {
        outside: outside.translation,
        inside_speak: speak.translation,
        inside_interact: interact.translation,
    });
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[type_path = "the_alchemist_assistant::npc"]
struct NpcArmature;

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
struct NpcAnimations {
    idle: AnimationNodeIndex,
    walk: AnimationNodeIndex,
}

#[derive(Component, Reflect, Default, Debug, Clone, PartialEq, Eq)]
#[reflect(Component)]
enum NpcAnimationState {
    #[default]
    Idle,
    Walking,
}

fn instance_ready_setup_armature(
    event: On<WorldInstanceReady>,
    mut commands: Commands,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    asset_server: Res<AssetServer>,
    armature: Single<Entity, With<NpcArmature>>,
) {
    let (graph, animations) = AnimationGraph::from_clips(
        (0..3).map(|i| asset_server.load(format!("Alchemist.glb#Animation{i}"))),
    );
    let graph_handle = graphs.add(graph);

    commands
        .entity(event.entity)
        .insert(NpcAnimationState::Idle);
    commands.entity(*armature).insert((
        AnimationGraphHandle(graph_handle),
        AnimationTransitions::new(),
        NpcAnimations {
            idle: animations[0],
            walk: animations[2],
        },
    ));
}

fn update_npc_animation(
    mut commands: Commands,
    mut npc: Single<(Entity, &mut NpcAnimationState, &Npc), Changed<Npc>>,
    mut armature: Single<(
        &NpcAnimations,
        &mut AnimationPlayer,
        &mut AnimationTransitions,
    )>,
) {
    let (entity, animation_state, npc) = &mut *npc;
    let new_animation = match npc.state {
        NpcState::Idle | NpcState::PreparingToSpeak { .. } | NpcState::SpeakingToPlayer(_) => {
            NpcAnimationState::Idle
        }
        NpcState::WalkIntoRoom(_) | NpcState::WalkOutOfRoom(_) => NpcAnimationState::Walking,
    };
    if new_animation != **animation_state {
        let (animations, animation_player, animation_transitions) = &mut *armature;
        let (animation, speed) = match new_animation {
            NpcAnimationState::Idle => {
                commands.entity(*entity).remove::<IsMoving>();
                (animations.idle, 1.)
            }
            NpcAnimationState::Walking => {
                commands
                    .entity(*entity)
                    .insert(IsMoving::default().with_volume(1.2));
                (animations.walk, 2.)
            }
        };
        **animation_state = new_animation;
        animation_transitions
            .play(animation_player, animation, Duration::from_millis(150))
            .set_speed(speed)
            .repeat();
    }
}

#[derive(Component, Reflect, Default, Clone)]
#[reflect(Component)]
struct NpcSpeechText;

fn npc_speech_scene(world: &mut DeferredWorld) -> impl Scene {
    // This is the texture that will be rendered to.
    let image = Image::new_target_texture(
        1024,
        512,
        bevy::render::render_resource::TextureFormat::Bgra8UnormSrgb,
        None,
    );

    let mut images = world.resource_mut::<Assets<Image>>();
    let image_handle = images.add(image);

    let texture_camera = world
        .commands()
        .spawn((
            Name::new("Npc speech camera"),
            Camera2d,
            Camera {
                // render before the "main pass" camera
                order: -1,
                clear_color: ClearColorConfig::Custom(Color::NONE),
                ..default()
            },
            bevy::camera::RenderTarget::Image(image_handle.clone().into()),
            DespawnOnExit(GameState::Game),
        ))
        .id();

    world.commands().spawn_scene(bsn! {
        #NpcSpeechNode
        Node {
            // Cover the whole image
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::End,
            align_items: AlignItems::Center,
        }
        UiTargetCamera(texture_camera)
        DespawnOnExit::<GameState>(GameState::Game)
        Children [
            #NpcSpeechText
            Text::new("")
            TextFont {
                font_size: px(32),
                weight: FontWeight::EXTRA_BOLD
            }
            NpcSpeechText
        ]
    });

    let mut meshes = world.resource_mut::<Assets<Mesh>>();
    let mesh_handle = meshes.add(Plane3d::new(Vec3::Z, Vec2::new(2., 1.)));

    let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(image_handle),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });

    bsn! {
        Children [
            Mesh3d(mesh_handle)
            MeshMaterial3d::<StandardMaterial>(material_handle)
            Billboarded
            Transform::from_xyz(0., 3.8, 0.)
        ]
    }
}

fn update_npc_speech_text(
    mut text: Single<&mut Text, With<NpcSpeechText>>,
    npc: Single<&Npc, Changed<Npc>>,
) {
    text.0 = match &npc.state {
        NpcState::SpeakingToPlayer(MessageToPlayer { message, .. }) => message.clone(),
        NpcState::PreparingToSpeak { .. } => String::from("..."),
        _ => String::new(),
    };
}
