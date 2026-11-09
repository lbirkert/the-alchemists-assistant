use bevy::audio::Volume;

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(GameUpdate, (add_cauldron_audio, update_cauldron_audio));
}

#[derive(Component, Reflect, Default, Clone)]
#[reflect(Component)]
enum CauldronAudio {
    #[default]
    Paused,
    FadeIn {
        timer: Timer,
    },
    FadeOut {
        timer: Timer,
    },
    Playing,
}

fn add_cauldron_audio(mut commands: Commands, added: Query<Entity, Added<Cauldron>>) {
    for entity in added {
        commands.entity(entity).apply_scene(bsn! {
            CauldronAudio
            AudioPlayer("audio/cauldron/cauldron_boiling.ogg")
            PlaybackSettings::LOOP.with_spatial(true).with_volume(Volume::Linear(0.))
        });
    }
}

const FADE_IN: Duration = Duration::from_millis(50);
const FADE_OUT: Duration = Duration::from_millis(150);

fn update_cauldron_audio(
    time: Res<Time>,
    query: Query<(&Cauldron, &mut CauldronAudio, &mut SpatialAudioSink)>,
) {
    for (cauldron, mut audio, mut sink) in query {
        let is_filled = !cauldron.items.is_empty();
        let is_paused = matches!(
            *audio,
            CauldronAudio::Paused | CauldronAudio::FadeOut { .. }
        );

        if is_filled && is_paused {
            sink.play();
            *audio = CauldronAudio::FadeIn {
                timer: Timer::new(FADE_IN, TimerMode::Once),
            };
        } else if !is_filled && !is_paused {
            *audio = CauldronAudio::FadeOut {
                timer: Timer::new(FADE_OUT, TimerMode::Once),
            };
        } else {
            let playback_volume = 1.;

            match &mut *audio {
                CauldronAudio::FadeIn { timer } => {
                    timer.tick(time.delta());
                    sink.set_volume(Volume::Linear(playback_volume * timer.fraction()));
                    if timer.is_finished() {
                        *audio = CauldronAudio::Playing;
                    }
                }
                CauldronAudio::FadeOut { timer } => {
                    timer.tick(time.delta());
                    sink.set_volume(Volume::Linear(playback_volume * (1. - timer.fraction())));
                    if timer.is_finished() {
                        sink.pause();
                        *audio = CauldronAudio::Paused;
                    }
                }
                _ => {}
            }
        }
    }
}
