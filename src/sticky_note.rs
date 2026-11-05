use bevy::{
    camera::RenderTarget, picking::backend::HitData, render::render_resource::TextureFormat,
};

use crate::{prelude::*, utils};

pub fn plugin(app: &mut App) {
    app.add_systems(GamePostUpdate, set_sticky_note_texture);
    app.add_systems(GameUpdate, pickup_sticky_note);
    app.add_systems(GameUpdate, place_sticky_note);
}

#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect(Default)]
#[reflect(Component)]
#[component(on_remove)]
pub struct NoteMaterial {
    image_handle: Option<Handle<Image>>,
    material_handle: Option<Handle<StandardMaterial>>,
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

        if let Some(material) = note_material.material_handle {
            commands.queue(move |world: &mut World| {
                world
                    .resource_mut::<Assets<StandardMaterial>>()
                    .remove(material.id());
            });
        }

        if let Some(image) = note_material.image_handle {
            commands.queue(move |world: &mut World| {
                world.resource_mut::<Assets<Image>>().remove(image.id());
            });
        }
    }
}

#[derive(Component, Reflect, Default, Debug, Clone, Eq, PartialEq, Hash)]
#[reflect(Component)]
#[reflect(Default)]
#[component(on_add)]
pub struct StickyNote {
    pub text: String,
    pub recipe: Option<Recipe>,
}

impl StickyNote {
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        let this = world.get::<Self>(ctx.entity).unwrap().clone();
        world
            .commands()
            .entity(ctx.entity)
            .insert(Interactable::StickyNote(this));
    }

    pub fn scene(self, asset_server: &AssetServer, pos: Vec3, rot: Quat) -> impl Scene {
        let scene = asset_server.load("StickyNotes.glb#Scene0");
        bsn! {
            #StickyNote
            WorldAssetRoot(scene)
            Transform::from_translation(pos).with_rotation(rot)
            self
        }
    }
}

#[derive(Component, Reflect, Default, Clone)]
#[reflect(Component)]
#[require(Interactable::StickyNotePlaceable)]
pub struct StickyNotePlaceable;

fn pickup_sticky_note(
    mut commands: Commands,
    mut messages: MessageReader<PlayerInteraction>,
    mut inventory: Single<&mut Inventory>,
) {
    let Some(PlayerInteraction {
        entity,
        kind: Interactable::StickyNote(note),
        ..
    }) = messages.read().next()
    else {
        return;
    };

    if inventory.selection.is_some() {
        return;
    }

    inventory.selection = Some(Selection::StickyNote(note.clone()));

    // destroy old sticky note
    commands.entity(*entity).despawn();
}

fn place_sticky_note(
    mut commands: Commands,
    mut messages: MessageReader<PlayerInteraction>,
    asset_server: Res<AssetServer>,
    mut inventory: Single<&mut Inventory>,
) {
    let Some(PlayerInteraction {
        kind: Interactable::StickyNotePlaceable,
        hit:
            HitData {
                position: Some(position),
                normal: Some(normal),
                ..
            },
        ..
    }) = messages.read().next()
    else {
        return;
    };

    if !inventory
        .selection
        .as_ref()
        .is_some_and(|i| i.is_sticky_node())
    {
        return;
    }

    let Some(Selection::StickyNote(sticky_note)) = inventory.selection.take() else {
        return;
    };
    let rotation = Transform::IDENTITY.looking_to(*normal, Vec3::Y).rotation;
    let scene = sticky_note.scene(&asset_server, *position, rotation);
    commands.spawn_scene(scene);
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
            continue;
        }

        let mut sticky_note = utils::find_parent::<StickyNote>(entity, &query)
            .and_then(|e| sticky_notes.get_mut(e).ok());

        let Some(ref mut note) = sticky_note else {
            continue;
        };

        let Some(mut material) = materials.get(material).cloned() else {
            continue;
        };

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
                        font_size: FontSize::Px(26.0),
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

        let material_handle = materials.add(material);
        note_material.material_handle = Some(material_handle.clone());
        commands
            .entity(entity)
            .insert(MeshMaterial3d(material_handle));
    }
}
