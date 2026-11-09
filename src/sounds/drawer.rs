use crate::{drawer::Drawer, prelude::*};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(GameUpdate, play_drawer_sound);
}

fn play_drawer_sound(
    mut messages: MessageReader<PlayerInteraction>,
    mut commands: Commands,
    drawers: Query<&Drawer>,
) {
    let Some(PlayerInteraction {
        entity,
        kind: Interactable::Drawer,
        ..
    }) = messages.read().next()
    else {
        return;
    };

    if let Ok(drawer) = drawers.get(*entity) {
        let sources = match drawer.opened {
            true => &CLOSE_SOURCES,
            false => &OPEN_SOURCES,
        };
        commands.spawn_scene(bsn! {
            @audio_scene(*entity, sources)
            PlaybackSettings {
                volume: bevy::audio::Volume::Linear(1.),
                speed: 2.2
            }
        });
    }
}

const OPEN_SOURCES: [&str; 4] = [
    "audio/drawer/drawer_open_1.ogg",
    "audio/drawer/drawer_open_2.ogg",
    "audio/drawer/drawer_open_3.ogg",
    "audio/drawer/drawer_open_4.ogg",
];

const CLOSE_SOURCES: [&str; 4] = [
    "audio/drawer/drawer_close_1.ogg",
    "audio/drawer/drawer_close_2.ogg",
    "audio/drawer/drawer_close_3.ogg",
    "audio/drawer/drawer_close_4.ogg",
];
