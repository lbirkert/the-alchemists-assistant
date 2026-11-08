use bevy::audio::AudioPlugin;

use crate::prelude::*;

mod camera;
mod cauldron;
mod debug;
mod drawer;
mod game_layer;
mod interaction;
mod inventory;
mod item;
mod liquid;
mod manager;
mod npc;
mod player;
mod potion;
mod prelude;
mod recipe;
mod scene;
mod settings;
mod sounds;
mod state;
mod sticky_note;
mod ui;
mod upgrades;
mod utils;

pub fn main() -> AppExit {
    let mut app = App::new();
    // Bevy plugins
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "The Alchemists Assistant".to_string(),
                    name: Some("the-alchemists-assistant".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            })
            .set(AudioPlugin {
                default_spatial_scale: bevy::audio::SpatialScale(Vec3::ONE * 0.3),
                ..Default::default()
            }),
    );
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
        interaction::plugin,
        item::plugin,
        drawer::plugin,
        upgrades::plugin,
        sticky_note::plugin,
    ));
    app.add_plugins((
        manager::plugin,
        npc::plugin,
        potion::plugin,
        cauldron::plugin,
        liquid::plugin,
    ));

    app.run()
}
