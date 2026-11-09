use std::time::Duration;

use bevy::audio::Volume;

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(GameUpdate, play_movement_sound);
}

#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct IsMoving {
    timer: Timer,
    volume: Volume,
}

impl Default for IsMoving {
    fn default() -> Self {
        let mut timer = Timer::new(Duration::from_millis(420), TimerMode::Repeating);
        timer.almost_finish();
        Self {
            timer,
            volume: Volume::Linear(1.),
        }
    }
}

impl IsMoving {
    pub fn with_volume(mut self, linear: f32) -> Self {
        self.volume = Volume::Linear(linear);
        self
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
            commands.spawn_scene(bsn! {
                @audio_scene(entity, &AUDIO_SOURCES)
                PlaybackSettings {
                    volume: {moving.volume}
                }
            });
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
