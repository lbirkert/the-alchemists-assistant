use std::time::Duration;

use bevy::audio::Volume;
use rand::seq::IndexedRandom;

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(GameRunning, play_movement_sound);
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
            commands.spawn_scene(audio_scene(entity));
        }
    }
}

fn audio_scene(parent: Entity) -> impl Scene {
    let mut rng = rand::rng();
    let source = *AUDIO_SOURCES
        .choose(&mut rng)
        .expect("the slice is not empty");
    bsn! {
        #MovementSound
        Transform
        ChildOf(parent)
        AudioPlayer(source)
        PlaybackSettings::DESPAWN.with_spatial(true).with_volume(Volume::Linear(0.5))
    }
}

const AUDIO_SOURCES: [&str; 5] = [
    "audio/foot_steps/foot_step1.ogg",
    "audio/foot_steps/foot_step2.ogg",
    "audio/foot_steps/foot_step3.ogg",
    "audio/foot_steps/foot_step4.ogg",
    "audio/foot_steps/foot_step5.ogg",
];
