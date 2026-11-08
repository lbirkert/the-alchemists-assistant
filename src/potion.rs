use bevy::picking::backend::HitData;

use crate::{item::ITEM_SIZE, prelude::*, scene::NpcTable};

pub fn plugin(app: &mut App) {
    app.add_systems(GameUpdate, fill_potion);
    app.add_systems(GameUpdate, label_potion.after(crate::item::pickup_item));
    app.add_systems(GameUpdate, submit_potion);
}

// a potion crafted by a user that might have a sticky note attached
#[derive(Reflect, Clone, Debug, Default, PartialEq, Eq, Hash)]
#[reflect(Default)]
pub struct PotionData {
    pub items: Vec<Item>,
    pub recipe: Option<Recipe>,
    // TODO: type: round, square, etc...
}

#[derive(Reflect, Debug, PartialEq, Eq, Hash)]
pub enum PotionReport {
    Valid,
    /// The contained items are correct, but in the wrong order.
    WrongOrder,
    Invalid {
        extra_items: Vec<ItemKind>,
        missing_items: Vec<ItemKind>,
        washed_but_should_not: Option<ItemKind>,
        not_washed_but_should_have: bool,
        /// Stores the item kind, the expected item state (from the recipe) and the actual
        /// item state.
        wrong_states: Vec<(ItemKind, ItemState, ItemState)>,
    },
}

impl PotionData {
    pub fn can_validate(&self) -> bool {
        self.recipe.is_some()
    }

    pub fn validate(&self) -> Option<PotionReport> {
        let recipe = self.recipe.as_ref()?.items();
        let report = if self.items == recipe {
            PotionReport::Valid
        } else {
            #[derive(Default)]
            struct ItemKindData {
                count: usize,
                washed: usize,
                states: HashMap<ItemState, usize>,
            }

            // Create a collection of the desired item counts
            let mut items: HashMap<ItemKind, ItemKindData> = HashMap::new();
            for item in recipe {
                let data = items.entry(item.kind).or_default();
                data.count += 1;
                if item.is_washed {
                    data.washed += 1;
                }
                *data.states.entry(item.state).or_default() += 1;
            }

            let mut washed_but_should_not = None;
            let mut extra_items = vec![];
            let mut item_state_not_in_recipe: Vec<(ItemKind, ItemState)> = vec![];
            for item in &self.items {
                match items.get_mut(&item.kind) {
                    Some(data) => {
                        if data.count == 0 {
                            // TODO! item kind was there, but not enough
                        } else {
                            data.count -= 1;
                        }
                        if item.is_washed {
                            if data.washed == 0 {
                                washed_but_should_not = Some(item.kind);
                            } else {
                                data.washed -= 1;
                            }
                        }
                        match data.states.get_mut(&item.state) {
                            Some(count) if *count > 0 => *count -= 1,
                            _ => {
                                item_state_not_in_recipe.push((item.kind, item.state));
                            }
                        }
                    }
                    None => {
                        extra_items.push(item.kind);
                    }
                }
            }

            let all_recipe_items_contained = items.values().all(|data| {
                data.count == 0 && data.washed == 0 && data.states.values().all(|count| *count == 0)
            });

            if all_recipe_items_contained && extra_items.is_empty() {
                PotionReport::WrongOrder
            } else {
                PotionReport::Invalid {
                    extra_items,
                    missing_items: items.iter().fold(vec![], |mut acc, (kind, data)| {
                        if data.count > 0 {
                            acc.push(*kind);
                        }
                        acc
                    }),
                    washed_but_should_not,
                    not_washed_but_should_have: items.values().any(|data| data.washed > data.count),
                    wrong_states: item_state_not_in_recipe.into_iter().fold(
                        vec![],
                        |mut acc, (kind, state)| {
                            if let Some((recipe_state, count)) =
                                items.get_mut(&kind).and_then(|data| {
                                    data.states.iter_mut().find(|(_state, count)| **count > 0)
                                })
                            {
                                *count -= 1;
                                acc.push((kind, state, *recipe_state));
                            }
                            acc
                        },
                    ),
                }
            }
        };
        Some(report)
    }

    pub fn color(&self) -> Color {
        crate::utils::items_to_color(&self.items)
    }
}

impl PotionReport {
    pub fn is_valid(&self) -> bool {
        *self == PotionReport::Valid
    }
}

fn fill_potion(
    mut messages: MessageReader<PlayerInteraction>,
    mut inventory: Single<&mut Inventory>,
    mut cauldron: Single<&mut Cauldron>,
) {
    let Some(PlayerInteraction {
        kind: Interactable::Cauldron,
        ..
    }) = messages.read().next()
    else {
        return;
    };

    if cauldron.is_empty() {
        return;
    }

    if !matches!(
        inventory.selection,
        Some(Selection::Item(Item {
            kind: ItemKind::Potion,
            potion: None,
            ..
        }))
    ) {
        return;
    }

    let potion = PotionData {
        items: cauldron.items.clone(),
        recipe: None,
    };
    cauldron.items.clear();
    inventory.selection.replace(Selection::Item(Item {
        kind: ItemKind::Potion,
        potion: Some(potion),
        ..Default::default()
    }));
}

fn label_potion(
    mut commands: Commands,
    mut messages: MessageReader<PlayerInteraction>,
    mut inventory: Single<&mut Inventory>,
) {
    let Some(PlayerInteraction {
        entity,
        // Careful with race pickup_item
        kind: Interactable::Item(item),
        ..
    }) = messages.read().next()
    else {
        return;
    };

    let mut item = item.clone();
    let Item {
        potion: Some(ref mut potion @ PotionData { recipe: None, .. }),
        ..
    } = item
    else {
        return;
    };

    if !matches!(inventory.selection, Some(Selection::StickyNote(..))) {
        return;
    };

    let Some(Selection::StickyNote(sticky_note)) = inventory.selection.take() else {
        panic!();
    };

    potion.recipe = sticky_note.recipe.clone();

    // update potion (so that sticky note shows)
    commands
        .entity(*entity)
        .insert((Interactable::Item(item.clone()), item));
}

fn submit_potion(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut messages: MessageReader<PlayerInteraction>,
    mut inventory: Single<&mut Inventory>,
    mut npc_table: Single<&mut NpcTable>,
) {
    let Some(PlayerInteraction {
        kind: Interactable::NpcTable,
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

    if !matches!(
        inventory.selection,
        Some(Selection::Item(Item {
            potion: Some(PotionData {
                recipe: Some(..),
                ..
            }),
            ..
        }))
    ) {
        return;
    };

    let Some(Selection::Item(item)) = inventory.selection.take() else {
        panic!();
    };

    // TODO: we should maybe move the potion to a predefined grid instead of taking hit position
    let scene = item.clone().scene(
        &asset_server,
        Transform::from_translation(*position + *normal * (ITEM_SIZE / 2.0)),
    );

    let potion_entity = commands.spawn_scene(scene).id();
    npc_table
        .potions
        .push((potion_entity, item.potion.unwrap()));
}
