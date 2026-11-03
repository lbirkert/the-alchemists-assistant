use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup);

    app.add_systems(OnEnter(GamePaused(false)), grab_cursor);
    app.add_systems(OnExit(GamePaused(false)), ungrab_cursor);
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

fn grab_cursor(mut primary_cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>) {
    primary_cursor_options.grab_mode = CursorGrabMode::Locked;
    primary_cursor_options.visible = false;
}

fn ungrab_cursor(mut primary_cursor_options: Single<&mut CursorOptions, With<PrimaryWindow>>) {
    primary_cursor_options.grab_mode = CursorGrabMode::None;
    primary_cursor_options.visible = true;
}
