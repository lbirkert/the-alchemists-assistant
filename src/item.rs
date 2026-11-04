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

impl std::fmt::Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Item::Garlic { state, is_washed } => {
                write!(f, "{}Garlic", Self::name_prefix(state, is_washed))
            }
            Item::Crystal { state, is_washed } => {
                write!(f, "{}Crystal", Self::name_prefix(state, is_washed))
            }
            Item::Flowers { state, is_washed } => {
                write!(f, "{}Flowers", Self::name_prefix(state, is_washed))
            }
            Item::Grass { state, is_washed } => {
                write!(f, "{}Grass", Self::name_prefix(state, is_washed))
            }
        }
    }
}

impl Item {
    fn name_prefix(state: &ItemState, is_washed: &bool) -> String {
        match (is_washed, state) {
            (false, ItemState::Unprocessed) => format!(""),
            (false, ItemState::Sliced) => format!("Sliced "),
            (false, ItemState::MortaredCoarse) => format!("Coarse-Mortared "),
            (false, ItemState::MortaredFine) => format!("Fine-Mortared "),
            (true, ItemState::Unprocessed) => format!("Washed "),
            (true, ItemState::Sliced) => format!("Washed Sliced "),
            (true, ItemState::MortaredCoarse) => format!("Washed Coarse-Mortared "),
            (true, ItemState::MortaredFine) => format!("Washed Fine-Mortared "),
        }
    }

    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        let this = *world.get::<Self>(ctx.entity).unwrap();
        world
            .commands()
            .entity(ctx.entity)
            .insert(Interactable::Item(this));
    }

    // returns a short description without the prefix
    pub fn short(&self) -> &'static str {
        match self {
            Item::Garlic { .. } => "Garlic",
            Item::Crystal { .. } => "Crystal",
            Item::Grass { .. } => "Grass",
            Item::Flowers { .. } => "Flowers",
        }
    }

    // state machines. Can be copied to mortar.rs and slicing.rs once implemented
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
