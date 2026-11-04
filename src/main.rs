use crate::prelude::*;

mod camera;
mod debug;
mod drawer;
mod game_layer;
mod inventory;
mod item;
mod player;
mod prelude;
mod recipe;
mod scene;
mod settings;
mod sounds;
mod state;
mod sticky_notes;
mod ui;
mod utils;

pub fn main() -> AppExit {
    for _ in 0..10 {
        let recipe = recipe::Recipe::generate();
        println!("----\n{}", recipe.task_description);
    }

    let mut app = App::new();
    // Bevy plugins
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "The Alchemists Assistant".to_string(),
            ..Default::default()
        }),
        ..Default::default()
    }));
    app.add_plugins(MeshPickingPlugin);
    app.insert_resource(MeshPickingSettings::default());

    // Ecosystem plugins
    app.add_plugins(PhysicsPlugins::default());

    app.add_plugins(bevy_skein::SkeinPlugin::default());

    app.add_plugins(bevy_easings::EasingsPlugin::default());

    // Our plugins
    app.add_plugins((
        state::plugin,
        camera::plugin,
        settings::plugin,
        utils::plugin,
        ui::plugin,
        debug::plugin,
        scene::plugin,
        sounds::plugin,
        player::plugin,
        inventory::plugin,
        drawer::plugin,
        sticky_notes::plugin,
    ));

    app.run()
}
