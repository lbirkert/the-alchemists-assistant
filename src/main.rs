use bevy::prelude::*;

use crate::state::state_plugin;

pub mod state;

pub fn main() -> AppExit {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "The Alchemists Assistant".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugins(state_plugin)
        // TODO:
        .run()
}
