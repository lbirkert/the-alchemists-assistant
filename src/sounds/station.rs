use bevy::audio::Volume;

use crate::{
    prelude::*,
    scene::stations::{ProcessingStation, Station, StationState},
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(GameUpdate, (add_station_audio, update_station_audio));
}

#[derive(Component, Reflect, Default, Clone)]
#[reflect(Component)]
enum StationAudio {
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

fn add_station_audio(
    mut commands: Commands,
    added: Query<(Entity, &ProcessingStation), Added<ProcessingStation>>,
) {
    for (entity, station) in added {
        let source = match station.kind {
            Station::CuttingBoard => CUT_SOURCE,
            Station::Mortar => PESTLE_SOURCE,
        };
        commands.entity(entity).apply_scene(bsn! {
            StationAudio
            AudioPlayer(source)
            PlaybackSettings::LOOP.with_spatial(true).with_volume(Volume::Linear(0.))
        });
    }
}

const FADE_IN: Duration = Duration::from_millis(50);
const FADE_OUT: Duration = Duration::from_millis(150);

fn update_station_audio(
    time: Res<Time>,
    query: Query<(&ProcessingStation, &mut StationAudio, &mut SpatialAudioSink)>,
) {
    for (station, mut audio, mut sink) in query {
        let is_processing = matches!(
            station.state,
            StationState::Processing { working: true, .. }
        );
        let is_paused = matches!(*audio, StationAudio::Paused | StationAudio::FadeOut { .. });

        if is_processing && is_paused {
            sink.play();
            *audio = StationAudio::FadeIn {
                timer: Timer::new(FADE_IN, TimerMode::Once),
            };
        } else if !is_processing && !is_paused {
            *audio = StationAudio::FadeOut {
                timer: Timer::new(FADE_OUT, TimerMode::Once),
            };
        } else {
            let playback_volume = match station.kind {
                Station::Mortar => 2.,
                Station::CuttingBoard => 1.,
            };

            match &mut *audio {
                StationAudio::FadeIn { timer } => {
                    timer.tick(time.delta());
                    sink.set_volume(Volume::Linear(playback_volume * timer.fraction()));
                    if timer.is_finished() {
                        *audio = StationAudio::Playing;
                    }
                }
                StationAudio::FadeOut { timer } => {
                    timer.tick(time.delta());
                    sink.set_volume(Volume::Linear(playback_volume * (1. - timer.fraction())));
                    if timer.is_finished() {
                        sink.pause();
                        *audio = StationAudio::Paused;
                    }
                }
                _ => {}
            }
        }
    }
}

const CUT_SOURCE: &str = "audio/cutting_board/cutting_1.ogg";
const _FINE_CUT_SOURCE: &str = "audio/cutting_board/cutting_2.ogg";

const PESTLE_SOURCE: &str = "audio/mortar/mortar_1.ogg";
