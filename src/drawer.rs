use std::time::Duration;

use bevy::picking::backend::HitData;

use crate::{prelude::*, utils};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(GamePostUpdate, toggle_drawer);
}

#[derive(Component, Reflect, Debug, Clone, Default)]
#[reflect(Component, Default)]
#[require(Interactable::Drawer)]
pub struct Drawer {
    pub opened: bool,
    #[reflect(ignore)]
    translation: Option<Vec3>,
}

fn toggle_drawer(
    mut commands: Commands,
    input: Res<ButtonInput<MouseButton>>,
    pointer: Single<&PointerInteraction, With<PlayerCursor>>,
    find_query: Query<(Option<&ChildOf>, Has<Drawer>)>,
    mut drawers: Query<(&Transform, &mut Drawer)>,
) {
    if !input.just_pressed(MouseButton::Right) {
        return;
    }

    if let Some((entity, HitData { .. })) = pointer.get_nearest_hit() {
        let Some(entity) = utils::find::<Drawer>(*entity, &find_query) else {
            return;
        };

        let Ok((transform, mut drawer)) = drawers.get_mut(entity) else {
            return;
        };

        let translation = drawer
            .translation
            .get_or_insert(transform.translation)
            .clone();

        let target = if drawer.opened {
            transform.with_translation(Vec3::new(0.0, 0.0, 0.0) + translation)
        } else {
            transform.with_translation(Vec3::new(0.7, 0.0, 0.0) + translation)
        };
        drawer.opened = !drawer.opened;

        commands.entity(entity).insert(transform.ease_to(
            target,
            EaseFunction::CubicInOut,
            EasingType::Once {
                duration: Duration::from_millis(300),
            },
        ));
    }
}
