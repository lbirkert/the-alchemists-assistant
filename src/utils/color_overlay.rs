use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, setup);
}

#[derive(Component, Reflect, Default, Clone, Copy)]
#[reflect(Component)]
pub struct ColorOverlay;

fn setup(mut commands: Commands) {
    commands.spawn_scene(bsn! {
        #ColorOverlay
        ColorOverlay
        Node {
            width: percent(100),
            height: percent(100),
        }
        Pickable::IGNORE
        ZIndex(2)
    });
}
