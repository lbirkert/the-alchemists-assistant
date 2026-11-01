use bevy::input::common_conditions::{input_just_pressed, input_toggle_active};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};

use crate::prelude::*;

const DEBUG_ENABLED_DEFAULT: bool = cfg!(debug_assertions);

pub(super) fn plugin(app: &mut App) {
    // Inspector
    app.add_plugins(EguiPlugin::default());
    app.add_plugins(
        WorldInspectorPlugin::default()
            .run_if(input_toggle_active(DEBUG_ENABLED_DEFAULT, KeyCode::F1)),
    );

    // Physics debug
    app.add_plugins(PhysicsDebugPlugin);
    app.add_systems(
        GameRunning,
        toggle_physics_gizmo.run_if(input_just_pressed(KeyCode::F3)),
    );
    app.insert_gizmo_config(
        PhysicsGizmos::default(),
        GizmoConfig {
            enabled: DEBUG_ENABLED_DEFAULT,
            ..Default::default()
        },
    );
}

fn toggle_physics_gizmo(mut store: ResMut<GizmoConfigStore>) {
    if let Some((config, _)) = store.get_config_mut::<PhysicsGizmos>() {
        config.enabled = !config.enabled;
    }
}
