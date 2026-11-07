use bevy::audio::Volume;

use crate::{
    npc::{Npc, NpcState},
    prelude::*,
    sounds::audio_scene,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(GameUpdate, (add_npc_audio, update_npc_audio));
}

#[derive(Component, Reflect, Default, Clone)]
#[reflect(Component)]
enum NpcAudio {
    #[default]
    Idle,
    Harrumph,
    MumbleFadeIn {
        timer: Timer,
    },
    Mumbling,
    MumbleFadeOut {
        timer: Timer,
    },
}

fn add_npc_audio(mut commands: Commands, added: Query<Entity, Added<Npc>>) {
    for entity in added {
        commands.entity(entity).apply_scene(bsn! {
            NpcAudio
            AudioPlayer("audio/npc/npc_mumble.ogg")
            PlaybackSettings::LOOP.with_spatial(true).with_volume(Volume::Linear(0.))
        });
    }
}

const FADE_IN: Duration = Duration::from_millis(50);
const FADE_OUT: Duration = Duration::from_millis(150);
const MUMBLING_VOLUME: f32 = 2.;

fn update_npc_audio(
    time: Res<Time>,
    // mut commands: Commands,
    query: Query<(Entity, &Npc, &mut NpcAudio, &mut SpatialAudioSink)>,
) {
    for (_npc_entity, npc, mut audio, mut sink) in query {
        if matches!(npc.state, NpcState::PreparingToSpeak { .. })
            && !matches!(*audio, NpcAudio::Harrumph)
        {
            // My ears are bleeding
            // commands.spawn_scene(bsn! {
            //     @audio_scene(npc_entity, &["audio/npc/npc_harrumph.ogg"])
            //     PlaybackSettings {
            //         volume: Volume::Linear(1.),
            //     }
            // });
            sink.set_volume(Volume::Linear(0.));
            sink.pause();
            continue;
        }
        let is_speaking = matches!(npc.state, NpcState::SpeakingToPlayer(_));
        let is_paused = matches!(*audio, NpcAudio::Idle | NpcAudio::MumbleFadeOut { .. });

        if is_speaking && is_paused {
            sink.play();
            *audio = NpcAudio::MumbleFadeIn {
                timer: Timer::new(FADE_IN, TimerMode::Once),
            };
        } else if !is_speaking && !is_paused {
            *audio = NpcAudio::MumbleFadeOut {
                timer: Timer::new(FADE_OUT, TimerMode::Once),
            };
        } else {
            match &mut *audio {
                NpcAudio::MumbleFadeIn { timer } => {
                    timer.tick(time.delta());
                    sink.set_volume(Volume::Linear(MUMBLING_VOLUME * timer.fraction()));
                    if timer.is_finished() {
                        *audio = NpcAudio::Mumbling;
                    }
                }
                NpcAudio::MumbleFadeOut { timer } => {
                    timer.tick(time.delta());
                    sink.set_volume(Volume::Linear(MUMBLING_VOLUME * (1. - timer.fraction())));
                    if timer.is_finished() {
                        sink.pause();
                        *audio = NpcAudio::Idle;
                    }
                }
                _ => {}
            }
        }
    }
}
