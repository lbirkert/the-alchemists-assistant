use bevy::{
    input::mouse::MouseMotion,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};

use crate::{inventory::Inventory, prelude::*, settings::MovementSettings};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(ON_GAME_ENTER, (spawn_player, initial_grab_cursor));

    app.add_systems(
        GameRunning,
        (
            cursor_grab,
            update_player_item_preview,
            player_look.before(player_move),
            player_move.in_set(PhysicsSystems::First),
        ),
    );
}

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

fn spawn_player(mut commands: Commands) {
    commands.spawn_scene(bsn! {
        #Player
        Player
        Inventory {
            item: Item::Herb { kind: HerbKind::Red }
        }
        Transform {
            translation: Vec3 {
                x: 2.,
                y: 1.
            }
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
            ShapeCaster {
                direction: {Dir3::from_xyz(0.0, 0.0, -1.0).unwrap()},
                max_hits: 2,
            }
            #PlayerCamera
            Camera3d
            PlayerEyes
            CollisionLayers::new(GameLayer::Default, [GameLayer::Pickable, GameLayer::Environment])
            Transform {
                translation: Vec3 {
                    y: 1.
                }
            }
            Children [
                #RightPlayerHand
                PlayerRightHand
                Transform::from_xyz(0.175, -0.4, -1.).with_scale(Vec3::splat(1.0))
            ]
        ]
    });
}

fn cursor_grab(
    keys: Res<ButtonInput<KeyCode>>,
    key_bindings: Res<KeyBindings>,
    primary_cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if keys.just_pressed(key_bindings.toggle_grab_cursor) {
        toggle_grab_cursor(primary_cursor_options);
    }
}

/// Grabs/ungrabs mouse cursor
fn toggle_grab_cursor(mut primary_cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>) {
    match primary_cursor_options.grab_mode {
        CursorGrabMode::None => {
            primary_cursor_options.grab_mode = CursorGrabMode::Locked;
            primary_cursor_options.visible = false;
        }
        _ => {
            primary_cursor_options.grab_mode = CursorGrabMode::None;
            primary_cursor_options.visible = true;
        }
    }
}

/// Grabs the cursor when game first starts
fn initial_grab_cursor(primary_cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>) {
    toggle_grab_cursor(primary_cursor_options);
}

/// Handles keyboard input and movement
fn player_move(
    keys: Res<ButtonInput<KeyCode>>,
    primary_cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>,
    settings: Res<MovementSettings>,
    key_bindings: Res<KeyBindings>,
    mut player_velocity: Single<&mut LinearVelocity, With<Player>>,
    camera_transform: Single<&Transform, With<PlayerEyes>>,
) {
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

    let (yaw, _pitch, _roll) = camera_transform.rotation.to_euler(EulerRot::YXZ);
    direction = Quat::from_rotation_y(yaw) * direction.normalize_or_zero();

    let v = direction * settings.speed;
    player_velocity.0.x = v.x;
    player_velocity.0.z = v.z;
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
    inventory: Single<&Inventory, Changed<Inventory>>,
    right_hand: Single<Entity, With<PlayerRightHand>>,
    children: Query<&Children>,
) {
    if let Some(item) = inventory.item {
        let scene = item.scene();
        commands.entity(*right_hand).apply_scene(bsn! {
            Children [
                @scene
            ]
        });
    } else {
        let Ok(children) = children.get(*right_hand) else {
            return;
        };

        for child in children {
            commands.entity(*child).despawn();
        }
    }
}
