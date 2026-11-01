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
        .add_systems(Startup, spawn_scene)
        // TODO:
        .run()
}

pub fn spawn_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 2.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Light
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4)),
    ));

    // Floor
    commands.spawn_scene(bsn! {
        Mesh3d(asset_value(Plane3d::new(
            Vec3::new(0.0, 1.0, 0.0),
            Vec2::new(10.0, 10.0),
        )))
        MeshMaterial3d::<StandardMaterial>(asset_value(Color::srgba(1.0, 0.0, 0.0, 1.0)))
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0))
    });

    // GLB model
    commands.spawn((
        WorldAssetRoot(
            asset_server.load(
                GltfAssetLabel::Scene(0)
                    .from_asset("models/LowPolyAlchemyMarket_v1.0/GLB/alchemy_table.glb"),
            ),
        ),
        Transform::from_scale(Vec3::splat(1.0)),
    ));
}
