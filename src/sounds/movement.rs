use std::time::Duration;

use crate::{prelude::*, sounds::audio_scene};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(GameUpdate, play_movement_sound);
}

#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct IsMoving {
    timer: Timer,
}

impl Default for IsMoving {
    fn default() -> Self {
        let mut timer = Timer::new(Duration::from_millis(420), TimerMode::Repeating);
        timer.almost_finish();
        Self { timer }
    }
}

fn play_movement_sound(
    mut commands: Commands,
    time: Res<Time>,
    moving: Query<(Entity, &mut IsMoving)>,
) {
    for (entity, mut moving) in moving {
        moving.timer.tick(time.delta());
        if moving.timer.is_finished() {
            commands.spawn_scene(audio_scene(entity, &AUDIO_SOURCES));
        }
    }
}

const AUDIO_SOURCES: [&str; 5] = [
    "audio/foot_steps/foot_step1.ogg",
    "audio/foot_steps/foot_step2.ogg",
    "audio/foot_steps/foot_step3.ogg",
    "audio/foot_steps/foot_step4.ogg",
    "audio/foot_steps/foot_step5.ogg",
];
