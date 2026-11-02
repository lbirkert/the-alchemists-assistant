use crate::prelude::*;

mod debug;
mod player;
mod prelude;
mod scene;
mod settings;
mod state;
mod utils;

pub fn main() -> AppExit {
    let mut app = App::new();
    // Bevy plugins
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "The Alchemists Assistant".to_string(),
            ..Default::default()
        }),
        ..Default::default()
    }));

    // Ecosystem plugins
    app.add_plugins(PhysicsPlugins::default());

    app.add_plugins(bevy_skein::SkeinPlugin::default());

    // Our plugins
    app.add_plugins((
        state::plugin,
        settings::plugin,
        utils::plugin,
        debug::plugin,
        scene::plugin,
        player::plugin,
    ));

    app.run()
}
