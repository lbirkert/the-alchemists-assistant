use bevy::prelude::*;

use crate::{camera::PlayerPlugin, state::state_plugin};

pub mod camera;
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
        .add_plugins(PlayerPlugin)
        .add_plugins(state_plugin)
        .add_systems(Startup, spawn_scene)
        // TODO:
        .run()
}

pub fn spawn_scene(mut commands: Commands, asset_server: Res<AssetServer>) {
    const FLOOR_SIZE: f32 = 7.2;
    const WALL_HEIGHT: f32 = 3.0;

    // Light (TODO: use a different, more realistic light source)
    commands.spawn((
        PointLight {
            intensity: 600000.0,
            ..Default::default()
        },
        Transform::from_translation(Vec3::new(0.0, WALL_HEIGHT - 0.2, 0.0)),
    ));

    // floor + ceil
    commands.spawn_scene(bsn! {
        Mesh3d(asset_value(Plane3d::new(
            Vec3::new(0.0, 1.0, 0.0),
            Vec2::new(FLOOR_SIZE / 2.0, FLOOR_SIZE / 2.0),
        )))
        MeshMaterial3d::<StandardMaterial>(asset_value(Color::srgba(0.6, 0.6, 0.6, 1.0)))
        Transform::from_translation(Vec3::new(0.0, 0.0, 0.0))
    });
    commands.spawn_scene(bsn! {
        Mesh3d(asset_value(Plane3d::new(
            Vec3::new(0.0, -1.0, 0.0),
            Vec2::new(FLOOR_SIZE / 2.0, FLOOR_SIZE / 2.0),
        )))
        MeshMaterial3d::<StandardMaterial>(asset_value(Color::srgba(0.6, 0.6, 0.6, 1.0)))
        Transform::from_translation(Vec3::new(0.0, WALL_HEIGHT, 0.0))
    });

    // walls
    commands.spawn_scene_list(bsn_list! {
        Mesh3d(asset_value(Plane3d::new(
            Vec3::new(1.0, 0.0, 0.0),
            Vec2::new(WALL_HEIGHT / 2.0, FLOOR_SIZE / 2.0),
        )))
        MeshMaterial3d::<StandardMaterial>(asset_value(Color::srgba(1.0, 1.0, 1.0, 1.0)))
        Transform::from_translation(Vec3::new(-FLOOR_SIZE / 2.0, WALL_HEIGHT / 2.0, 0.0))
        --
        Mesh3d(asset_value(Plane3d::new(
            Vec3::new(-1.0, 0.0, 0.0),
            Vec2::new(WALL_HEIGHT / 2.0, FLOOR_SIZE / 2.0),
        )))
        MeshMaterial3d::<StandardMaterial>(asset_value(Color::srgba(1.0, 1.0, 1.0, 1.0)))
        Transform::from_translation(Vec3::new(FLOOR_SIZE / 2.0, WALL_HEIGHT / 2.0, 0.0))
        --
        Mesh3d(asset_value(Plane3d::new(
            Vec3::new(0.0, 0.0, 1.0),
            Vec2::new(FLOOR_SIZE / 2.0, WALL_HEIGHT / 2.0),
        )))
        MeshMaterial3d::<StandardMaterial>(asset_value(Color::srgba(1.0, 1.0, 1.0, 1.0)))
        Transform::from_translation(Vec3::new(0.0, WALL_HEIGHT / 2.0, -FLOOR_SIZE / 2.0))
        --
        Mesh3d(asset_value(Plane3d::new(
            Vec3::new(0.0, 0.0, -1.0),
            Vec2::new(FLOOR_SIZE / 2.0, WALL_HEIGHT / 2.0),
        )))
        MeshMaterial3d::<StandardMaterial>(asset_value(Color::srgba(1.0, 1.0, 1.0, 1.0)))
        Transform::from_translation(Vec3::new(0.0, WALL_HEIGHT / 2.0, FLOOR_SIZE / 2.0))
    });

    // tables
    commands.spawn_scene_list(bsn_list! {
        @table_model(&asset_server)
        Transform::from_translation(Vec3::new(FLOOR_SIZE / 2.0 - 3.2, 0.0, -FLOOR_SIZE / 2.0 + 0.5))
        --
        @table_model(&asset_server)
        Transform::from_translation(Vec3::new(FLOOR_SIZE / 2.0 - 5.2, 0.0, -FLOOR_SIZE / 2.0 + 0.5))
        --
        @table_model(&asset_server)
        Transform::from_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2))
            .with_translation(Vec3::new(-FLOOR_SIZE / 2.0 + 0.5, 0.0, FLOOR_SIZE / 2.0 - 5.2))
        --
        @table_model(&asset_server)
        Transform::from_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2))
            .with_translation(Vec3::new(-FLOOR_SIZE / 2.0 + 0.5, 0.0, FLOOR_SIZE / 2.0 - 3.2))
        --
        @table_model(&asset_server)
        Transform::from_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2))
            .with_translation(Vec3::new(-FLOOR_SIZE / 2.0 + 0.5, 0.0, FLOOR_SIZE / 2.0 - 1.2))
    });

    // barrel
    let barrel_asset = asset_server.load(
        GltfAssetLabel::Scene(0).from_asset("models/LowPolyAlchemyMarket_v1.0/GLB/barrel.glb"),
    );
    commands.spawn_scene(bsn! {
        WorldAssetRoot(barrel_asset)
        Transform::from_translation(Vec3::new(-FLOOR_SIZE / 2.0 + 0.5, 0.0, -FLOOR_SIZE / 2.0 + 0.5))
            .with_scale(Vec3::new(1.1, 1.1, 1.1))
    });

    // spawn cauldron
    let cauldron_asset = asset_server.load(
        GltfAssetLabel::Scene(0).from_asset("models/LowPolyAlchemyMarket_v1.0/GLB/cauldron.glb"),
    );
    commands.spawn_scene(bsn! {
        WorldAssetRoot(cauldron_asset)
        Transform::from_scale(Vec3::new(1.1, 1.1, 1.1)).with_translation(Vec3::new(0.0, 0.01, 0.0))
    });
    commands.spawn((
        PointLight {
            color: Srgba::rgb(0.0, 1.0, 0.0).into(),
            intensity: 800000.0,
            range: 2.0,
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.01, 0.0)),
    ));

    // spawn cabinet
    let cabinet_asset = asset_server.load(
        GltfAssetLabel::Scene(0)
            .from_asset("models/LowPolyAlchemyMarket_v1.0/GLB/apothecary_cabinet.glb"),
    );
    commands.spawn_scene(bsn! {
        WorldAssetRoot(cabinet_asset)
        Transform::from_scale(Vec3::new(1.5, 1.5, 1.5))
            .with_translation(Vec3::new(FLOOR_SIZE / 2.0 - 1.0, 0.0, -FLOOR_SIZE / 2.0 + 0.5))
    });

    // spawn mortar + pestle
    let mortar_asset = asset_server.load(
        GltfAssetLabel::Scene(0)
            .from_asset("models/LowPolyAlchemyMarket_v1.0/GLB/mortar_pestle.glb"),
    );
    commands.spawn_scene(bsn! {
        WorldAssetRoot(mortar_asset)
        Transform::from_scale(Vec3::new(0.8, 0.8, 0.8))
            .with_rotation(Quat::from_rotation_y(-std::f32::consts::FRAC_PI_4))
            .with_translation(Vec3::new(FLOOR_SIZE / 2.0 - 5.2, 0.95, -FLOOR_SIZE / 2.0 + 0.5))
    });

    // spawn scale
    let scales_asset = asset_server.load(
        GltfAssetLabel::Scene(0).from_asset("models/LowPolyAlchemyMarket_v1.0/GLB/scales.glb"),
    );
    commands.spawn_scene(bsn! {
        WorldAssetRoot(scales_asset)
        Transform::from_scale(Vec3::new(1.0, 1.0, 1.0))
            .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2))
            .with_translation(Vec3::new(-FLOOR_SIZE / 2.0 + 0.5, 0.95, FLOOR_SIZE / 2.0 - 5.2))
    });
}

fn table_model(asset_server: &Res<AssetServer>) -> impl Scene {
    let table_asset = asset_server.load(
        GltfAssetLabel::Scene(0)
            .from_asset("models/LowPolyAlchemyMarket_v1.0/GLB/alchemy_table.glb"),
    );
    bsn! {
        WorldAssetRoot(table_asset)
    }
}
