use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup);

    app.add_systems(ON_GAME_ENTER, grab_cursor);

    app.add_systems(GameUpdate, cursor_grab);
}

fn setup(mut commands: Commands) {
    commands.spawn_scene(bsn! {
        #PlayerCamera
        Camera3d
        MeshPickingCamera
        Transform {
            translation: Vec3 {
                y: 1.
            }
        }
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
fn toggle_grab_cursor(primary_cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>) {
    match primary_cursor_options.grab_mode {
        CursorGrabMode::None => {
            grab_cursor(primary_cursor_options);
        }
        _ => {
            ungrab_cursor(primary_cursor_options);
        }
    }
}

fn grab_cursor(mut primary_cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>) {
    primary_cursor_options.grab_mode = CursorGrabMode::Locked;
    primary_cursor_options.visible = false;
}

fn ungrab_cursor(mut primary_cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>) {
    primary_cursor_options.grab_mode = CursorGrabMode::None;
    primary_cursor_options.visible = true;
}
