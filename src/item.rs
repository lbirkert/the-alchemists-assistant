use crate::prelude::*;

pub const ITEM_SIZE: f32 = 0.3;

#[derive(Component, Default, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(Interactable::ItemPlaceable)]
pub struct ItemPlaceable;

#[derive(Component, Default, Reflect, Debug, Clone)]
#[reflect(Component)]
#[require(Interactable::ItemDiscardable)]
pub struct ItemDiscardable;

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[component(on_add)]
pub struct ItemSource {
    pub item: Item,
}

impl ItemSource {
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        let item = world.get::<Self>(ctx.entity).unwrap().item.clone();
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
pub struct Item {
    pub kind: ItemKind,
    pub state: ItemState,
    pub is_washed: bool,
    pub potion: Option<Potion>,
    // ^ don't like this tbh
    // we probably want to have a unspecific ItemKind and then a
    // different enum Item which contains the variants with data.
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

    pub fn scene(self, asset_server: &AssetServer, pos: Vec3) -> impl Scene {
        let asset = match self.kind {
            ItemKind::Garlic => match self.state {
                ItemState::Unprocessed => "Garlic.glb",
                ItemState::Sliced => "SlicedGarlic.glb",
                ItemState::Grinded => "GrindedGarlic.glb",
                ItemState::FineGrinded => "FineGrindedGarlic.glb",
            },
            ItemKind::Grass => "Grass.glb",
            ItemKind::Flowers => "Flowers.glb",
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
            Transform::from_translation(pos).with_scale(Vec3::splat(ITEM_SIZE))
        }
    }

    pub fn physics_scene(self, asset_server: &AssetServer, pos: Vec3, vel: Vec3) -> impl Scene {
        let scene = self.clone().scene(asset_server, pos);
        bsn! {
            RigidBody::Dynamic
            LinearVelocity(vel)
            Collider::cuboid(1.0, 1.0, 1.0)
            @scene
            Interactable::Item({self.clone()})
            self
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
