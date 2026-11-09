use bevy::picking::backend::HitData;

use crate::{prelude::*, scene::stations::Station};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(GameUpdate, (pickup_item, update_recipe_for_potion).chain());
    app.add_systems(GameUpdate, pickup_item_from_source);
    app.add_systems(GameUpdate, place_item);
    app.add_systems(GameUpdate, discard_item);
}

pub const ITEM_SIZE: f32 = 0.3;

#[derive(Component, Default, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(Interactable::ItemPlaceable)]
#[type_path = "the_alchemist_assistant::item"]
pub struct ItemPlaceable;

#[derive(Component, Default, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(Interactable::ItemDiscardable)]
#[type_path = "the_alchemist_assistant::item"]
pub struct ItemDiscardable;

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[component(on_add)]
#[type_path = "the_alchemist_assistant::item"]
pub struct ItemSource {
    pub item: Item,
}

impl ItemSource {
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        let item = world.get::<Self>(ctx.entity).unwrap().item.clone();

        // Sometimes bevy_skein behaves in unexpected ways.
        debug_assert!(item.kind == ItemKind::Potion || item.potion.is_none());

        world
            .commands()
            .entity(ctx.entity)
            .insert(Interactable::Source { item });
    }
}

// we might want to use https://github.com/FraserLee/bevy_sprite3d/blob/main/examples/sprite.rs
// for rendering the items. -> Ask faervan

#[derive(Component, Reflect, Debug, Default, Clone, PartialEq, Eq, Hash)]
#[reflect(Component)]
#[reflect(Default)]
#[component(on_add)]
pub struct Item {
    pub kind: ItemKind,
    pub state: ItemState,
    pub is_washed: bool,
    pub potion: Option<PotionData>,
}

#[derive(Component, Reflect, EnumIter, Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[reflect(Component)]
pub enum ItemKind {
    #[default]
    Garlic,
    Crystal,
    Grass,
    Flowers,
    Potion,
}

#[derive(Reflect, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemState {
    #[default]
    Unprocessed,
    Sliced,
    Grinded,
    FineGrinded,
}

impl std::fmt::Display for ItemKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Garlic => "Garlic",
                Self::Crystal => "Crystal",
                Self::Grass => "Grass",
                Self::Flowers => "Flowers",
                Self::Potion => "Potion",
            }
        )
    }
}

impl std::fmt::Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.name_prefix(), self.kind)
    }
}

impl Item {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let entity = context.entity;

        if let Some(Item {
            kind: ItemKind::Potion,
            potion,
            ..
        }) = world.get::<Item>(entity)
        {
            let color = potion.as_ref().map(|p| p.color()).unwrap_or(Color::BLACK);
            let is_visible = potion.is_some();
            world
                .commands()
                .entity(entity)
                .insert(LiquidController { color, is_visible });
        }
    }

    fn name_prefix(&self) -> String {
        match (self.is_washed, self.state) {
            (false, ItemState::Unprocessed) => format!(""),
            (false, ItemState::Sliced) => format!("Sliced "),
            (false, ItemState::Grinded) => format!("Grinded "),
            (false, ItemState::FineGrinded) => format!("Fine-Grinded "),
            (true, ItemState::Unprocessed) => format!("Washed "),
            (true, ItemState::Sliced) => format!("Washed Sliced "),
            (true, ItemState::Grinded) => format!("Washed Coarse-Mortared "),
            (true, ItemState::FineGrinded) => format!("Washed Fine-Mortared "),
        }
    }

    pub fn scene(self, asset_server: &AssetServer, transform: Transform) -> impl Scene {
        let asset = match self.kind {
            ItemKind::Garlic => match self.state {
                ItemState::Unprocessed => "Garlic.glb",
                ItemState::Sliced => "SlicedGarlic.glb",
                ItemState::Grinded => "GrindedGarlic.glb",
                ItemState::FineGrinded => "FineGrindedGarlic.glb",
            },
            ItemKind::Grass => "Grass.glb",
            ItemKind::Flowers => match self.state {
                ItemState::Unprocessed => "Flowers.glb",
                ItemState::Sliced => "SlicedFlowers.glb",
                ItemState::Grinded => panic!(),
                ItemState::FineGrinded => "FineGrindedFlowers.glb",
            },
            ItemKind::Potion => "Potion.glb",
            ItemKind::Crystal => match self.state {
                ItemState::Unprocessed => "Crystal.glb",
                ItemState::Grinded => "GrindedCrystal.glb",
                ItemState::FineGrinded => "FineGrindedCrystal.glb",
                ItemState::Sliced => panic!(),
            },
        };

        let asset = asset_server.load(GltfAssetLabel::Scene(0).from_asset(asset));

        bsn! {
            self
            WorldAssetRoot(asset)
            transform.with_scale(Vec3::splat(ITEM_SIZE))
        }
    }

    pub fn physics_scene(
        self,
        asset_server: &AssetServer,
        transform: Transform,
        vel: Vec3,
    ) -> impl Scene {
        let scene = self.clone().scene(asset_server, transform);
        bsn! {
            RigidBody::Dynamic
            LinearVelocity(vel)
            Collider::cuboid(1.0, 1.0, 1.0)
            @scene
            Interactable::Item({self})
        }
    }

    /// Set the state to sliced.
    pub fn sliced(mut self) -> Self {
        self.state = ItemState::Sliced;
        self
    }

    /// Set the state to grinded.
    pub fn grinded(mut self) -> Self {
        self.state = ItemState::Grinded;
        self
    }

    /// Set the state to fine grinded.
    pub fn fine_grinded(mut self) -> Self {
        self.state = ItemState::FineGrinded;
        self
    }

    /// Set the state to washed.
    pub fn washed(mut self) -> Self {
        self.is_washed = true;
        self
    }
}

impl ItemState {
    pub fn needs_station(&self) -> Option<Station> {
        match self {
            Self::Unprocessed => None,
            Self::Sliced => Some(Station::CuttingBoard),
            Self::Grinded | Self::FineGrinded => Some(Station::Mortar),
        }
    }
}

fn pickup_item_from_source(
    mut messages: MessageReader<PlayerInteraction>,
    mut commands: Commands,
    mut inventory: Single<&mut Inventory>,
) {
    let Some(PlayerInteraction {
        kind: Interactable::Source { item: source },
        entity,
        ..
    }) = messages.read().next()
    else {
        return;
    };

    if inventory.selection.is_some() {
        return;
    }

    let audio_sources: &[&str] = match source.kind {
        ItemKind::Potion => &[
            "audio/potion/potion_take_1.ogg",
            "audio/potion/potion_take_2.ogg",
            "audio/potion/potion_take_3.ogg",
        ],
        _ => &[
            "audio/item/item_pickup_1.ogg",
            "audio/item/item_pickup_2.ogg",
            "audio/item/item_pickup_3.ogg",
            "audio/item/item_pickup_4.ogg",
        ],
    };
    commands.spawn_scene(audio_scene(*entity, audio_sources));

    inventory.selection = Some(Selection::Item(source.clone()));
}

fn update_recipe_for_potion(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    items: Query<(Entity, &Item), Changed<Item>>,
    sticky_notes: Query<(Entity, &ChildOf), With<StickyNote>>,
) {
    for (entity, item) in &items {
        // Remove existing sticky notes
        for (note_entity, child_of) in &sticky_notes {
            if child_of.parent() == entity {
                commands.entity(note_entity).despawn();
            }
        }

        let Some(PotionData {
            recipe: Some(ref recipe),
            ..
        }) = item.potion
        else {
            continue;
        };

        let sticky_note_scene = StickyNote {
            text: recipe.task_description.clone(),
            recipe: Some(recipe.clone()),
        }
        .scene(
            &asset_server,
            Transform::from_translation(Vec3::new(0.0, 0.25, -0.3))
                .with_scale(Vec3::splat(0.6))
                .with_rotation(Quat::from_rotation_x(0.5)),
        );

        commands.spawn_scene(bsn! {
            @sticky_note_scene
            ChildOf(entity)
        });
    }
}

pub fn pickup_item(
    mut messages: MessageReader<PlayerInteraction>,
    mut commands: Commands,
    assets_server: Res<AssetServer>,
    mut inventory: Single<&mut Inventory>,
    transforms: Query<&Transform>,
) {
    let Some(PlayerInteraction {
        entity,
        // Careful with race condition with label_potion!
        kind: Interactable::Item(item),
        ..
    }) = messages.read().next()
    else {
        return;
    };

    if inventory.selection.as_ref().is_some_and(|s| !s.is_item()) {
        return;
    }

    info!("picking up, with item in hand: {:?}", inventory.selection);

    if let Ok(item_transform) = transforms.get(*entity) {
        let old_item = inventory.selection.replace(Selection::Item(item.clone()));
        match old_item {
            Some(Selection::Item(old_item)) => {
                let item_scene =
                    old_item.physics_scene(&assets_server, *item_transform, Vec3::ZERO);
                commands.spawn_scene(item_scene);
            }
            Some(Selection::StickyNote(_)) => unreachable!("We checked this above"),
            None => {}
        }
        // Destroy the focused item, we already put it in the inventory
        commands.entity(*entity).despawn();

        let audio_sources: &[&str] = match item.kind {
            ItemKind::Potion => &[
                "audio/potion/potion_take_1.ogg",
                "audio/potion/potion_take_2.ogg",
                "audio/potion/potion_take_3.ogg",
            ],
            _ => &[
                "audio/item/item_pickup_1.ogg",
                "audio/item/item_pickup_2.ogg",
                "audio/item/item_pickup_3.ogg",
                "audio/item/item_pickup_4.ogg",
            ],
        };
        commands.spawn_scene(audio_scene_at(item_transform.translation, audio_sources));
    }
}

fn place_item(
    mut messages: MessageReader<PlayerInteraction>,
    mut commands: Commands,
    assets_server: Res<AssetServer>,
    mut inventory: Single<&mut Inventory>,
) {
    let Some(PlayerInteraction {
        kind: Interactable::ItemPlaceable,
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

    if !inventory.selection.as_ref().is_some_and(|s| s.is_item()) {
        return;
    }

    let Some(Selection::Item(item)) = inventory.selection.take() else {
        return;
    };

    let audio_sources: &[&str] = match item.kind {
        ItemKind::Potion => &[
            "audio/potion/potion_place_1.ogg",
            "audio/potion/potion_place_2.ogg",
        ],
        ItemKind::Crystal => &[
            "audio/item/item_place_crystal_1.ogg",
            "audio/item/item_place_crystal_2.ogg",
            "audio/item/item_place_crystal_3.ogg",
            "audio/item/item_place_crystal_4.ogg",
            "audio/item/item_place_crystal_5.ogg",
        ],
        _ => &[
            "audio/item/item_place_1.ogg",
            "audio/item/item_place_2.ogg",
            "audio/item/item_place_3.ogg",
            "audio/item/item_place_4.ogg",
            "audio/item/item_place_5.ogg",
        ],
    };
    commands.spawn_scene(audio_scene_at(*position, audio_sources));

    let item_scene = item.physics_scene(
        &assets_server,
        Transform::from_translation(position + normal * ITEM_SIZE / 2.0),
        Vec3::ZERO,
    );
    commands.spawn_scene(item_scene);
}

fn discard_item(
    mut messages: MessageReader<PlayerInteraction>,
    mut inventory: Single<&mut Inventory>,
) {
    let Some(PlayerInteraction {
        kind: Interactable::ItemDiscardable,
        ..
    }) = messages.read().next()
    else {
        return;
    };

    if !inventory.selection.as_ref().is_some_and(|s| s.is_item()) {
        return;
    }

    inventory.selection = None;
}
