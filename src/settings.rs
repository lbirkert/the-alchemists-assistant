use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<MovementSettings>();
    app.init_resource::<KeyBindings>();
}

/// Mouse sensitivity and movement speed
#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct MovementSettings {
    pub sensitivity: f32,
    pub speed: f32,
    pub lr_speed_modifier: f32,
    pub sprint_speed_modifier: f32,
}

impl Default for MovementSettings {
    fn default() -> Self {
        Self {
            sensitivity: 0.00012,
            speed: 7.,
            sprint_speed_modifier: 1.5,
            lr_speed_modifier: 0.7,
        }
    }
}

/// Key configuration
#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct KeyBindings {
    pub move_forward: KeyCode,
    pub move_backward: KeyCode,
    pub move_left: KeyCode,
    pub move_right: KeyCode,
    pub move_ascend: KeyCode,
    pub move_descend: KeyCode,
    pub move_sprint: KeyCode,
    pub pause_game: KeyCode,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            move_forward: KeyCode::KeyW,
            move_backward: KeyCode::KeyS,
            move_left: KeyCode::KeyA,
            move_right: KeyCode::KeyD,
            move_ascend: KeyCode::Space,
            move_descend: KeyCode::ShiftLeft,
            move_sprint: KeyCode::ShiftLeft,
            pause_game: KeyCode::Escape,
        }
    }
}
