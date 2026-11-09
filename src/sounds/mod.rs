use bevy::audio::Volume;

use crate::prelude::*;

mod cauldron;
mod drawer;
pub mod movement;
mod npc;
mod station;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        movement::plugin,
        drawer::plugin,
        station::plugin,
        npc::plugin,
        cauldron::plugin,
    ));
}

pub fn audio_scene(parent: Entity, sources: &[&'static str]) -> impl Scene {
    let mut rng = rand::rng();
    let source = *sources.choose(&mut rng).expect("No audio sources");
    bsn! {
        #Sound
        Transform
        ChildOf(parent)
        AudioPlayer(source)
        PlaybackSettings::DESPAWN.with_spatial(true).with_volume(Volume::Linear(0.5))
    }
}

pub fn audio_scene_at(pos: Vec3, sources: &[&'static str]) -> impl Scene {
    let mut rng = rand::rng();
    let source = *sources.choose(&mut rng).expect("No audio sources");
    bsn! {
        #Sound
        Transform::from_translation(pos)
        AudioPlayer(source)
        PlaybackSettings::DESPAWN.with_spatial(true).with_volume(Volume::Linear(0.5))
    }
}

pub fn audio_scene_not_spatial(sources: &[&'static str]) -> impl Scene {
    let mut rng = rand::rng();
    let source = *sources.choose(&mut rng).expect("No audio sources");
    bsn! {
        #Sound
        AudioPlayer(source)
        PlaybackSettings::DESPAWN.with_volume(Volume::Linear(0.5))
    }
}
