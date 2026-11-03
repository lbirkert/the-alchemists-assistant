use bevy::{picking::hover::Hovered, ui::Pressed, ui_widgets::Activate};

use crate::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        add_focusable_item_component.run_if(resource_added::<FocusableItems>),
    );
    app.add_systems(
        // Needs to be post update, see <https://github.com/vleue/bevy_easings/issues/23>
        PostUpdate,
        update_items.run_if(resource_exists::<FocusableItems>),
    );
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub(super) struct FocusableItems {
    focused: usize,
    pub items: Vec<Entity>,
}

impl FocusableItems {
    pub fn new(items: impl Into<Vec<Entity>>) -> Self {
        let items = items.into();
        assert_ne!(items.len(), 0);
        Self { focused: 0, items }
    }
}

#[derive(EntityEvent)]
pub(super) struct FocusUpdate {
    pub entity: Entity,
    pub is_focused: bool,
}

#[derive(Component, FromTemplate, Reflect, Clone, Copy, Default)]
#[reflect(Component)]
#[require(Hovered)]
struct FocusableItem {
    already_active: bool,
    ignore_hover: bool,
}

fn add_focusable_item_component(mut commands: Commands, items: Res<FocusableItems>) {
    for entity in &items.items {
        commands.entity(*entity).insert(FocusableItem::default());
    }
}

fn update_items(
    mut commands: Commands,
    mut menu: ResMut<FocusableItems>,
    input: Res<ButtonInput<KeyCode>>,
    query: Query<(Entity, &mut FocusableItem, &Hovered, Has<Pressed>)>,
) {
    if input.any_just_pressed([
        KeyCode::KeyW,
        KeyCode::KeyA,
        KeyCode::ArrowUp,
        KeyCode::KeyK,
    ]) || (input.pressed(KeyCode::ShiftLeft) && input.just_pressed(KeyCode::Tab))
    {
        // Previous
        if menu.focused == 0 {
            menu.focused = menu.items.len() - 1;
        } else {
            menu.focused -= 1;
        }
    } else if input.any_just_pressed([
        KeyCode::KeyS,
        KeyCode::KeyD,
        KeyCode::ArrowDown,
        KeyCode::KeyJ,
    ]) || input.just_pressed(KeyCode::Tab)
    {
        // Next
        menu.focused = (menu.focused + 1) % menu.items.len();
    }

    for (entity, mut button, hovered, is_pressed) in query {
        let is_active = menu.items[menu.focused] == entity
            || (hovered.0 && !button.ignore_hover && !button.already_active)
            || is_pressed;

        button.ignore_hover = hovered.0 && !is_active;

        if is_active != button.already_active {
            button.already_active = is_active;

            commands.entity(entity).trigger(|e| FocusUpdate {
                entity: e,
                is_focused: is_active,
            });

            if is_active {
                menu.focused = menu
                    .items
                    .iter()
                    .position(|e| *e == entity)
                    .unwrap_or_default();
            }
        }
    }

    // Activate the focused button
    if input.any_just_pressed([KeyCode::KeyE, KeyCode::Enter]) {
        commands
            .entity(menu.items[menu.focused])
            .trigger(|e| Activate { entity: e });
    }
}
