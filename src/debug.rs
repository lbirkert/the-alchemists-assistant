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
        GameUpdate,
        toggle_physics_gizmo.run_if(input_just_pressed(KeyCode::F3)),
    );
    app.insert_gizmo_config(
        PhysicsGizmos::default(),
        GizmoConfig {
            enabled: DEBUG_ENABLED_DEFAULT,
            ..Default::default()
        },
    );

    // Pointer debug
    app.add_systems(
        GameUpdate,
        draw_mesh_intersections.run_if(input_toggle_active(DEBUG_ENABLED_DEFAULT, KeyCode::F4)),
    );
}

fn toggle_physics_gizmo(mut store: ResMut<GizmoConfigStore>) {
    if let Some((config, _)) = store.get_config_mut::<PhysicsGizmos>() {
        config.enabled = !config.enabled;
    }
}

/// see <https://docs.rs/bevy/0.20.1/bevy/picking/pointer/struct.PointerInteraction.html#scraped-examples>
fn draw_mesh_intersections(
    pointers: Query<&PointerInteraction, With<PlayerCursor>>,
    mut gizmos: Gizmos,
) {
    for (point, normal) in pointers
        .iter()
        .filter_map(|interaction| interaction.get_nearest_hit())
        .filter_map(|(_entity, hit)| hit.position.zip(hit.normal))
    {
        gizmos.sphere(point, 0.05, bevy::color::palettes::tailwind::RED_500);
        gizmos.arrow(
            point,
            point + normal.normalize() * 0.5,
            bevy::color::palettes::tailwind::PINK_100,
        );
    }
}
