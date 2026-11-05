use bevy::{
    camera::RenderTarget, picking::backend::HitData, render::render_resource::TextureFormat,
};

use crate::{prelude::*, utils};

pub fn plugin(app: &mut App) {
    app.insert_resource(CurrentRecipes {
        recipes: Vec::new(),
    });
    app.add_systems(GameUpdate, update_whiteboard);
    app.add_systems(GameUpdate, set_sticky_note_texture);
    app.add_systems(GameUpdate, pickup_sticky_note);
    app.add_systems(GameUpdate, place_sticky_note);
}

// TODO: remove bc obsolete
#[derive(Component, Reflect, Default, Debug, Clone, Copy)]
#[reflect(Component)]
pub struct Whiteboard;

#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect(Default)]
#[reflect(Component)]
#[component(on_remove)]
pub struct NoteMaterial {
    image_handle: Option<Handle<Image>>,
    texture_camera: Option<Entity>,
    texture_content: Option<Entity>,
}

impl NoteMaterial {
    fn on_remove(mut world: DeferredWorld, context: HookContext) {
        let Some(note_material) = world.get::<NoteMaterial>(context.entity).cloned() else {
            return;
        };

        let mut commands = world.commands();

        // despawn camera and content
        if let Some(camera) = note_material.texture_camera {
            commands.entity(camera).despawn();
        }

        if let Some(content) = note_material.texture_content {
            commands.entity(content).despawn();
        }

        // remove material
        if let Some(image) = note_material.image_handle {
            commands.queue(move |world: &mut World| {
                world.resource_mut::<Assets<Image>>().remove(image.id());
            });
        }
    }
}

#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect(Component)]
#[reflect(Default)]
#[component(on_add)]
pub struct StickyNote {
    text: String,
}

impl StickyNote {
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        let this = world.get::<Self>(ctx.entity).unwrap().clone();
        world
            .commands()
            .entity(ctx.entity)
            .insert(Interactable::StickyNote(this));
    }

    pub fn scene(self, asset_server: &AssetServer, pos: Vec3, normal: Vec3) -> impl Scene {
        let scene = asset_server.load("StickyNotes.glb#Scene0");
        bsn! {
            #StickyNote
            WorldAssetRoot(scene)
            Transform::from_translation(pos).looking_to(normal, Vec3::Y)
            self
        }
    }
}

#[derive(Component, Reflect, Default, Clone)]
#[reflect(Component)]
#[require(Interactable::PlaceableLocation)]
pub struct StickyNotesPlaceable;

#[derive(Resource)]
pub struct CurrentRecipes {
    pub recipes: Vec<Recipe>,
}

// TODO: deprecate this in favor of a proper day management system (+ npc)
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

    let scene = asset_server.load("StickyNotes.glb#Scene0");
    commands.entity(whiteboard).with_children(|parent| {
        parent.spawn((
            Name::new("StickyNote"),
            StickyNote { text },
            WorldAssetRoot(scene),
            Transform::from_xyz(-0.03, 0.0, 0.01).looking_to(Vec3::NEG_X, Vec3::Y),
        ));
    });
}

fn pickup_sticky_note(
    mut commands: Commands,
    // assets_server: Res<AssetServer>,
    input: Res<ButtonInput<MouseButton>>,
    mut inventory: Single<&mut Inventory>,
    pointer: Single<&PointerInteraction, With<PlayerCursor>>,
    query: Query<(Option<&ChildOf>, Has<StickyNote>)>,
    note_query: Query<(&StickyNote, &Transform)>,
) {
    if !input.just_pressed(MouseButton::Right) {
        return;
    }

    if inventory.selection.is_some() {
        return;
    }

    if let Some((mesh_entity, _hit)) = pointer.get_nearest_hit()
        && let Some(entity) = crate::utils::find::<StickyNote>(*mesh_entity, &query)
        && let Ok((note, _note_transform)) = note_query.get(entity)
    {
        inventory.selection = Some(Selection::StickyNote(note.clone()));

        // destroy old sticky note
        commands.entity(entity).despawn();
    }
}

fn place_sticky_note(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    input: Res<ButtonInput<MouseButton>>,
    mut inventory: Single<&mut Inventory>,
    pointer: Single<&PointerInteraction, With<PlayerCursor>>,
    locations: Query<&Transform, With<StickyNotesPlaceable>>,
) {
    if !input.just_pressed(MouseButton::Right) {
        return;
    }

    if !inventory
        .selection
        .as_ref()
        .is_some_and(|i| i.is_sticky_node())
    {
        return;
    }

    if let Some((
        entity,
        HitData {
            position: Some(position),
            normal: Some(normal),
            ..
        },
    )) = pointer.get_nearest_hit()
    {
        if locations.get(*entity).is_err() {
            return;
        }

        let Some(Selection::StickyNote(sticky_note)) = inventory.selection.take() else {
            return;
        };

        let scene = sticky_note.scene(&asset_server, *position, *normal);
        commands.spawn_scene(scene);
    }
}

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

            let texture_content = commands
                .spawn((
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
                            font_size: FontSize::Px(28.0),
                            weight: FontWeight(900),
                            font: FontSource::Handle(font),
                            ..default()
                        },
                        TextColor(Color::BLACK)
                    )],
                ))
                .id();

            note_material.image_handle = Some(image_handle.clone());
            note_material.texture_camera = Some(texture_camera);
            note_material.texture_content = Some(texture_content);

            material.base_color_texture = Some(image_handle);
        }
    }
}
