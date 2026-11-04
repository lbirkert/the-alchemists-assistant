use bevy::{camera::RenderTarget, render::render_resource::TextureFormat};

use crate::{prelude::*, utils};

// TODO: remove bc obsolete
#[derive(Component, Reflect, Default, Debug, Clone, Copy)]
#[reflect(Component)]
pub struct Whiteboard;

#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect(Component)]
pub struct NoteMaterial {
    image_handle: Option<Handle<Image>>,
}

#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect(Component)]
#[reflect(Default)]
pub struct StickyNote {
    text: String,
}

#[derive(Component, Reflect, Default, Clone)]
#[reflect(Component)]
pub struct StickyNotePlaceable;

#[derive(Resource)]
pub struct CurrentRecipes {
    pub recipes: Vec<Recipe>,
}

pub fn plugin(app: &mut App) {
    app.insert_resource(CurrentRecipes {
        recipes: Vec::new(),
    });
    app.add_systems(GameUpdate, update_whiteboard);
    app.add_systems(GameUpdate, set_sticky_note_texture);
    app.add_systems(GameUpdate, pickup_sticky_note);
}

fn update_whiteboard(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    whiteboard_query: Query<Entity, With<Whiteboard>>,
    mut current_recipes: ResMut<CurrentRecipes>,
) {
    let Ok(whiteboard) = whiteboard_query.single() else {
        return;
    };

    if current_recipes.recipes.len() != 0 {
        return;
    }

    let text = Recipe::generate().task_description;
    current_recipes.recipes.push(Recipe::generate());

    info!("Spawn sticky note!");

    let scene = asset_server.load("StickyNotes.glb#Scene0");

    commands.entity(whiteboard).with_children(|parent| {
        parent.spawn((
            Name::new("StickyNote"),
            StickyNote { text },
            WorldAssetRoot(scene),
            Transform::from_xyz(0.0, 0.0, 0.01),
        ));
    });
}

fn pickup_sticky_note(
    mut commands: Commands,
    // assets_server: Res<AssetServer>,
    input: Res<ButtonInput<MouseButton>>,
    mut inventory: Single<&mut Inventory>,
    pointer: Single<&PointerInteraction, With<PlayerCursor>>,
    query: Query<(Option<&ChildOf>, Has<Item>)>,
    note_query: Query<(&StickyNote, &Transform)>,
) {
    if !input.just_pressed(MouseButton::Right) {
        return;
    }

    if inventory.selection.is_some() {
        return;
    }

    if let Some((mesh_entity, _hit)) = pointer.get_nearest_hit()
        && let Some(entity) = crate::utils::find(*mesh_entity, &query)
        && let Ok((note, _note_transform)) = note_query.get(entity)
    {
        inventory.selection = Some(Selection::StickyNote(note.clone()));

        // destroy old sticky note
        commands.entity(entity).despawn();
    }
}

// fn place_sticky_note(
//     mut commands: Commands,
//     assets_server: Res<AssetServer>,
//     input: Res<ButtonInput<MouseButton>>,
//     mut inventory: Single<&mut Inventory>,
//     pointer: Single<&PointerInteraction, With<PlayerCursor>>,
//     locations: Query<&Transform, With<StickyNotePlaceable>>,
// ) {
//     if !input.just_pressed(MouseButton::Right) {
//         return;
//     }
//
//     if let Some((
//         entity,
//         HitData {
//             position: Some(position),
//             normal: Some(normal),
//             ..
//         },
//     )) = pointer.get_nearest_hit()
//     {
//         if locations.get(*entity).is_err() {
//             return;
//         }
//
//         let Some(item) = inventory.item.take() else {
//             return;
//         };
//
//         let item_scene = item.physics_scene(
//             &assets_server,
//             position + normal * ITEM_SIZE / 2.0,
//             Vec3::ZERO,
//         );
//         commands.spawn_scene(item_scene);
//     }
// }

fn set_sticky_note_texture(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,

    mut sticky_notes: Query<&mut StickyNote>,
    note_materials: Query<(Entity, &MeshMaterial3d<StandardMaterial>, &mut NoteMaterial)>,
    query: Query<(Option<&ChildOf>, Has<StickyNote>)>,

    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    for (entity, material, mut note_material) in note_materials {
        if note_material.image_handle.is_some() {
            return;
        }

        let mut sticky_note =
            utils::find::<StickyNote>(entity, &query).and_then(|e| sticky_notes.get_mut(e).ok());

        let Some(ref mut note) = sticky_note else {
            return;
        };

        if let Some(mut material) = materials.get_mut(material) {
            // Create a texture to render into.
            let image = Image::new_target_texture(512, 512, TextureFormat::Bgra8UnormSrgb, None);

            let image_handle = images.add(image);

            // Camera rendering into the texture.
            let texture_camera = commands
                .spawn((
                    Camera2d,
                    Camera {
                        order: -1,
                        ..default()
                    },
                    RenderTarget::Image(image_handle.clone().into()),
                ))
                .id();

            let font: Handle<Font> = asset_server.load("FirstTimeWriting.ttf");

            // UI rendered into the texture.
            commands.spawn((
                Node {
                    width: percent(100),
                    height: percent(100),
                    justify_content: JustifyContent::Center,
                    padding: UiRect::all(percent(5)),
                    align_items: AlignItems::Start,
                    ..default()
                },
                BackgroundColor(Color::srgb(1.0, 1.0, 0.5)),
                UiTargetCamera(texture_camera),
                children![(
                    Text::new(note.text.clone()),
                    TextFont {
                        font_size: FontSize::Px(26.0),
                        font: FontSource::Handle(font),
                        ..default()
                    },
                    TextColor(Color::BLACK)
                )],
            ));

            info!("render text {}", note.text);

            note_material.image_handle = Some(image_handle.clone());

            material.base_color_texture = Some(image_handle);
        }
    }
}
