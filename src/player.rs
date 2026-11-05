use bevy::{
    camera::RenderTarget,
    input::mouse::MouseMotion,
    picking::pointer::{Location, PointerId, PointerLocation},
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};

use crate::{
    inventory::Inventory, prelude::*, settings::MovementSettings, sounds::movement::IsMoving,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(ON_GAME_ENTER, spawn_player);
    app.add_systems(ON_GAME_EXIT, despawn_player);

    app.add_systems(
        GameUpdate,
        (
            add_pointer,
            update_pointer_pos,
            update_player_item_preview,
            fix_fall_to_void,
            player_look.before(player_move),
            player_move.in_set(PhysicsSystems::First),
        ),
    );
}

const PLAYER_SPAWN_POS: Vec3 = Vec3::new(2., 2., 0.);

#[derive(Component, Reflect, Default, Clone, Copy)]
#[reflect(Component)]
struct Player;

#[derive(Component, Reflect, Default, Clone, Copy)]
#[reflect(Component)]
/// Marker component for the camera
struct PlayerEyes;

#[derive(Component, Reflect, Default, Clone, Copy)]
#[reflect(Component)]
struct PlayerRightHand;

fn spawn_player(mut commands: Commands, camera: Single<Entity, With<Camera3d>>) {
    let player_entity = commands.spawn_scene(bsn! {
        #Player
        Player
        Inventory
        Transform {
            translation: PLAYER_SPAWN_POS
        }
        Visibility::Inherited
        RigidBody::Dynamic
        Collider::capsule(0.4, 1.8)
        DebugRender::none()
        LockedAxes::ROTATION_LOCKED
        CollisionLayers::new(GameLayer::Player, [GameLayer::Default, GameLayer::Pickable, GameLayer::Environment])
        Mass(10.)
        NoAutoMass
        Children [
            #PlayerEars
            // Space between the ears
            SpatialListener::new(0.5)
            DespawnOnExit::<GameState>(GameState::Game)
        ]
    }).id();

    commands.entity(*camera).apply_scene(bsn! {
        ChildOf(player_entity)
        PlayerEyes
        Children [
            #RightPlayerHand
            PlayerRightHand
            Visibility::Inherited
            Transform::from_xyz(0.175, -0.25, -1.)
            DespawnOnExit::<GameState>(GameState::Game)
        ]
    });
}

fn despawn_player(mut commands: Commands, player: Single<Entity, With<Player>>) {
    commands.entity(*player).detach_all_children().despawn();
}

fn fix_fall_to_void(mut transform: Single<&mut Transform, With<Player>>) {
    if transform.translation.y < -100. {
        transform.translation = PLAYER_SPAWN_POS;
    }
}

#[derive(Component, Reflect, Default, Clone, Copy)]
#[reflect(Component)]
pub struct PlayerCursor;

fn add_pointer(
    mut commands: Commands,
    camera: Single<(&Camera, &RenderTarget), Added<Camera3d>>,
    window: Single<Entity, With<PrimaryWindow>>,
) {
    let (camera, target) = *camera;
    let Some(size) = camera.logical_viewport_size() else {
        error!("Failed to get logical viewport size");
        return;
    };
    let location = PointerLocation {
        location: Some(Location {
            position: size / 2.,
            target: target
                .normalize(Some(*window))
                .expect("Current implementation can't be None if input is Some"),
        }),
    };

    commands.spawn((
        Name::new("PlayerPointer"),
        PlayerCursor,
        PointerId::Custom(bevy::asset::uuid::Uuid::new_v4()),
        location,
    ));
}

fn update_pointer_pos(
    camera: Single<&Camera, (Changed<Camera>, With<Camera3d>)>,
    mut cursor: Single<&mut PointerLocation, With<PlayerCursor>>,
) {
    let Some(size) = camera.logical_viewport_size() else {
        error!("Failed to get logical viewport size");
        return;
    };
    if let Some(location) = &mut cursor.location {
        location.position = size / 2.;
    }
}

/// Handles keyboard input and movement
fn player_move(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    primary_cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>,
    settings: Res<MovementSettings>,
    key_bindings: Res<KeyBindings>,
    mut player_velocity: Single<(Entity, &mut LinearVelocity, Has<IsMoving>), With<Player>>,
    camera_transform: Single<&Transform, With<PlayerEyes>>,
) {
    let (entity, velocity, has_moving) = &mut *player_velocity;
    let mut direction = Vec3::ZERO;

    for key in keys.get_pressed() {
        match primary_cursor_options.grab_mode {
            CursorGrabMode::None => (),
            _ => {
                let key = *key;
                if key == key_bindings.move_forward {
                    direction += Vec3::NEG_Z;
                } else if key == key_bindings.move_backward {
                    direction += Vec3::Z;
                } else if key == key_bindings.move_left {
                    direction += Vec3::NEG_X;
                } else if key == key_bindings.move_right {
                    direction += Vec3::X;
                }
            }
        }
    }

    if direction != Vec3::ZERO && !*has_moving {
        commands.entity(*entity).insert(IsMoving::default());
    } else if direction == Vec3::ZERO && *has_moving {
        commands.entity(*entity).remove::<IsMoving>();
    }

    let (yaw, _pitch, _roll) = camera_transform.rotation.to_euler(EulerRot::YXZ);
    direction = Quat::from_rotation_y(yaw) * direction.normalize_or_zero();

    let v = direction * settings.speed;
    velocity.0.x = v.x;
    velocity.0.z = v.z;
}

/// Handles looking around if cursor is locked
fn player_look(
    settings: Res<MovementSettings>,
    primary_window: Query<&mut Window, With<PrimaryWindow>>,
    primary_cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>,
    mut state: MessageReader<MouseMotion>,
    query: Query<&mut Transform, With<PlayerEyes>>,
) {
    if let Ok(window) = primary_window.single() {
        for mut transform in query {
            for ev in state.read() {
                let (mut yaw, mut pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
                match primary_cursor_options.grab_mode {
                    CursorGrabMode::None => (),
                    _ => {
                        // Using smallest of height or width ensures equal vertical and horizontal sensitivity
                        let window_scale = window.height().min(window.width());
                        pitch -= (settings.sensitivity * ev.delta.y * window_scale).to_radians();
                        yaw -= (settings.sensitivity * ev.delta.x * window_scale).to_radians();
                    }
                }

                pitch = pitch.clamp(-1.54, 1.54);

                // Order is important to prevent unintended roll
                transform.rotation =
                    Quat::from_axis_angle(Vec3::Y, yaw) * Quat::from_axis_angle(Vec3::X, pitch);
            }
        }
    } else {
        warn!("Primary window not found for `player_look`!");
    }
}

fn update_player_item_preview(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    inventory: Single<&Inventory, Changed<Inventory>>,
    right_hand: Single<Entity, With<PlayerRightHand>>,
    children: Query<&Children>,
) {
    match inventory.selection.as_ref().cloned() {
        Some(Selection::Item(item)) => {
            let scene = item.scene(&asset_server, Vec3::ZERO);
            commands.entity(*right_hand).apply_scene(bsn! {
                Children [
                    @scene
                ]
            });
        }
        Some(Selection::StickyNote(sticky_note)) => {
            let scene = sticky_note.scene(
                &asset_server,
                Vec3::ZERO,
                Quat::from_rotation_y(std::f32::consts::PI),
            );
            commands.entity(*right_hand).apply_scene(bsn! {
                Children [
                @scene
            ]
            });
        }
        None => {
            let Ok(children) = children.get(*right_hand) else {
                return;
            };

            for child in children {
                commands.entity(*child).despawn();
            }
        }
    };
}
