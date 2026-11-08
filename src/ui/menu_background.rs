use crate::{
    npc::{Npc, NpcAnimations, NpcRightHand},
    prelude::*,
};

pub fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Title), spawn_scene);
    app.add_systems(Update, rotate_camera.run_if(in_state(GameState::Title)));
    app.add_systems(Update, set_npc_animation.run_if(in_state(GameState::Title)));
    app.add_systems(
        Update,
        bind_mixer_to_npc_hand.run_if(in_state(GameState::Title)),
    );
}

#[derive(Component, Reflect, Debug, Clone, Default)]
#[reflect(Component)]
struct MenuBackgroundCamera;

#[derive(Component, Reflect, Debug, Clone, Default)]
#[reflect(Component)]
struct Mixer;

fn spawn_scene(mut commands: Commands) {
    commands.spawn_scene(bsn! {
        #GameScene
        WorldAssetRoot("TheAlchemistsAssistant.glb#Scene0")
        DespawnOnExit<GameState>(GameState::Title)
    });

    commands.spawn_scene(bsn! {
        #Mixer
        Mixer
        WorldAssetRoot("Mixer.glb#Scene0")
        DespawnOnExit<GameState>(GameState::Title)
    });

    commands.spawn_scene(bsn! {
        #Camera
        MenuBackgroundCamera
        Camera3d
        Camera {
            order: 1,
        }
        Transform::from_translation(Vec3::splat(0.0))
        DespawnOnExit<GameState>(GameState::Title)
    });

    commands.insert_resource(NpcInitialized(false));

    commands.insert_resource(GlobalAmbientLight {
        color: Color::srgba(1.0, 1.0, 0.8, 1.0),
        brightness: 100.0,
        ..default()
    });
}

#[derive(Resource, Default)]
struct NpcInitialized(bool);

fn bind_mixer_to_npc_hand(
    global_transform: Single<&GlobalTransform, With<NpcRightHand>>,
    mut transform: Single<&mut Transform, With<Mixer>>,
) {
    transform.translation = global_transform.translation();
}

fn set_npc_animation(
    mut commands: Commands,
    mut armature: Single<(
        &NpcAnimations,
        &mut AnimationPlayer,
        &mut AnimationTransitions,
    )>,
    mut npc_initialized: ResMut<NpcInitialized>,
) {
    if npc_initialized.0 {
        return;
    }

    let (animations, animation_player, animation_transitions) = &mut *armature;
    let (animation, speed) = (animations.stir, 0.5);
    animation_transitions
        .play(animation_player, animation, Duration::from_millis(150))
        .set_speed(speed)
        .repeat();
    commands.run_system_cached(set_npc_pos);
    npc_initialized.0 = true;
}

fn set_npc_pos(mut npc_transform: Single<&mut Transform, With<Npc>>) {
    *npc_transform.as_mut() = Transform::from_xyz(0.0, 0.0, -1.5);
}

fn rotate_camera(
    mut transform: Single<&mut Transform, With<MenuBackgroundCamera>>,
    time: Res<Time>,
) {
    let (rsin, rcos) = (time.elapsed_secs() * 0.1).sin_cos();
    let r = 4.5;

    transform.translation = Vec3::new(rsin * r, 3.0, rcos * r);
    transform.look_at(Vec3::new(0.0, 1.5, 0.0), Dir3::Y);
}
