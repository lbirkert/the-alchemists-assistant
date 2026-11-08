use std::time::Duration;

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(GamePostUpdate, toggle_drawer);
}

#[derive(Component, Reflect, Debug, Clone, Default)]
#[reflect(Component, Default)]
#[require(Interactable::Drawer)]
#[type_path = "the_alchemist_assistant::drawer"]
pub struct Drawer {
    pub opened: bool,
    #[reflect(ignore)]
    translation: Option<Vec3>,
}

fn toggle_drawer(
    mut messages: MessageReader<PlayerInteraction>,
    mut commands: Commands,
    mut drawers: Query<(&Transform, &mut Drawer)>,
) {
    let Some(PlayerInteraction {
        entity,
        kind: Interactable::Drawer,
        ..
    }) = messages.read().next()
    else {
        return;
    };

    let Ok((transform, mut drawer)) = drawers.get_mut(*entity) else {
        return;
    };

    let translation = *drawer.translation.get_or_insert(transform.translation);

    let target = if drawer.opened {
        transform.with_translation(Vec3::new(0.0, 0.0, 0.0) + translation)
    } else {
        transform.with_translation(Vec3::new(0.7, 0.0, 0.0) + translation)
    };
    drawer.opened = !drawer.opened;

    commands.entity(*entity).insert(transform.ease_to(
        target,
        EaseFunction::CubicInOut,
        EasingType::Once {
            duration: Duration::from_millis(300),
        },
    ));
}
