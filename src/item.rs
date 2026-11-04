use crate::prelude::*;

pub const ITEM_SIZE: f32 = 0.3;

#[derive(Component, Default, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(Interactable::PlaceableLocation)]
pub struct ItemsPlaceable;

#[derive(Component, Default, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(Interactable::Discardable)]
pub struct ItemsDiscardable;

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[component(on_add)]
pub struct ItemsSource {
    pub item: Item,
}

impl ItemsSource {
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        let item = world.get::<Self>(ctx.entity).unwrap().item;
        world
            .commands()
            .entity(ctx.entity)
            .insert(Interactable::Source { item });
    }
}

// we might want to use https://github.com/FraserLee/bevy_sprite3d/blob/main/examples/sprite.rs
// for rendering the items. -> Ask faervan

#[derive(Component, Reflect, Debug, Clone, Copy)]
#[reflect(Component)]
#[component(on_add)]
pub enum Item {
    Garlic { state: ItemState, is_washed: bool },
    Crystal { state: ItemState, is_washed: bool },
    Grass { state: ItemState, is_washed: bool },
    Flowers { state: ItemState, is_washed: bool },
}

#[derive(Reflect, Debug, Clone, Copy)]
pub enum ItemState {
    Unprocessed,
    Sliced,
    MortaredCoarse,
    MortaredFine,
}

#[derive(Reflect, Debug, Clone, Copy)]
pub enum MortarState {
    None,
    Coarse,
    Fine,
}

impl Item {
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        let this = *world.get::<Self>(ctx.entity).unwrap();
        world
            .commands()
            .entity(ctx.entity)
            .insert(Interactable::Item(this));
    }

    pub fn mortar(self) -> Item {
        match self {
            Item::Garlic {
                state: ItemState::Unprocessed,
                is_washed,
            } => Item::Garlic {
                state: ItemState::MortaredCoarse,
                is_washed,
            },
            Item::Garlic {
                state: ItemState::Sliced | ItemState::MortaredCoarse,
                is_washed,
            } => Item::Garlic {
                state: ItemState::MortaredFine,
                is_washed,
            },
            _ => self,
        }
    }

    pub fn slice(self) -> Item {
        match self {
            Item::Garlic {
                state: ItemState::Unprocessed,
                is_washed,
            } => Item::Garlic {
                state: ItemState::Sliced,
                is_washed,
            },
            _ => self,
        }
    }

    pub fn scene(self, asset_server: &AssetServer, pos: Vec3) -> impl Scene {
        let asset = match self {
            Item::Garlic { state, .. } => match state {
                ItemState::Unprocessed => "Garlic.glb",
                ItemState::Sliced => "GarlicSliced.glb",
                ItemState::MortaredCoarse => "GarlicMortaredCoarse.glb",
                ItemState::MortaredFine => "GarlicMortaredFine.glb",
            },
            Item::Grass { .. } => "Grass.glb",
            Item::Flowers { .. } => "Flowers.glb",
            Item::Crystal { .. } => "Crystal.glb",
        };

        let asset = asset_server.load(GltfAssetLabel::Scene(0).from_asset(asset));

        bsn! {
            self
            WorldAssetRoot(asset)
            Transform::from_translation(pos).with_scale(Vec3::splat(ITEM_SIZE))
        }
    }

    pub fn physics_scene(self, asset_server: &AssetServer, pos: Vec3, vel: Vec3) -> impl Scene {
        let scene = self.scene(asset_server, pos);
        bsn! {
            RigidBody::Dynamic
            LinearVelocity(vel)
            Collider::cuboid(1.0, 1.0, 1.0)
            @scene
        }
    }
}
