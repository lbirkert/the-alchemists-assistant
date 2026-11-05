use crate::{
    item::ItemsSource,
    prelude::*,
    scene::stations::{ProcessingStation, Station},
};

pub(super) fn plugin(app: &mut App) {
    app.add_message::<NewUnlock>();

    app.init_resource::<Statistics>();
    app.init_resource::<GameUpgrades>();

    app.add_systems(
        GameUpdate,
        (
            hide_locked_things,
            check_upgrade_requirements.run_if(resource_changed::<Statistics>),
        ),
    );
}

#[derive(Resource, Reflect, Default, Debug)]
#[reflect(Resource)]
pub struct Statistics {
    items_processed: usize,
    potions_brewed: usize,
    item_counts: HashMap<Item, usize>,
}

impl Statistics {
    pub fn process_item(&mut self, item: Item) {
        self.items_processed += 1;
        *self.item_counts.entry(item).or_default() += 1;
    }
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
struct GameUpgrades {
    items: HashMap<Item, UpgradeState>,
    stations: HashMap<Station, UpgradeState>,
}

#[derive(Reflect)]
enum UpgradeState {
    UnlockedByDefault,
    Locked { requirements: Vec<UnlockCondition> },
    Unlocked,
}

#[derive(Message, Clone, Copy, Debug)]
pub enum NewUnlock {
    Item(Item),
    Station(Station),
}

#[derive(Reflect)]
enum UnlockCondition {
    TotalItemsProcessed(usize),
    TotalPotionsBrewed(usize),
    ItemsProcessed { item: Item, min_processed: usize },
}

impl UnlockCondition {
    fn item(item: Item, min: usize) -> Self {
        Self::ItemsProcessed {
            item,
            min_processed: min,
        }
    }

    fn is_satisfied(&self, statistics: &Statistics) -> bool {
        match self {
            Self::TotalItemsProcessed(n) => statistics.items_processed >= *n,
            Self::TotalPotionsBrewed(n) => statistics.potions_brewed >= *n,
            Self::ItemsProcessed {
                item,
                min_processed,
            } => statistics
                .item_counts
                .get(item)
                .is_some_and(|n| n >= min_processed),
        }
    }
}

impl Default for GameUpgrades {
    fn default() -> Self {
        let items = Item::iter()
            .map(|item| {
                (
                    item,
                    match item {
                        Item::Garlic => UpgradeState::UnlockedByDefault,
                        Item::Flowers => UpgradeState::UnlockedByDefault,
                        Item::Grass => UpgradeState::Locked {
                            requirements: vec![UnlockCondition::item(Item::Flowers, 3)],
                        },
                        Item::Crystal => UpgradeState::Locked {
                            requirements: vec![UnlockCondition::TotalItemsProcessed(10)],
                        },
                    },
                )
            })
            .collect();
        let stations = Station::iter()
            .map(|item| {
                (
                    item,
                    match item {
                        Station::CuttingBoard => UpgradeState::UnlockedByDefault,
                        Station::Mortar => UpgradeState::Locked {
                            requirements: vec![UnlockCondition::TotalItemsProcessed(5)],
                        },
                    },
                )
            })
            .collect();
        Self { items, stations }
    }
}

impl GameUpgrades {
    fn item_is_locked(&self, item: &Item) -> bool {
        matches!(self.items[item], UpgradeState::Locked { .. })
    }

    fn station_is_locked(&self, station: &Station) -> bool {
        matches!(self.stations[station], UpgradeState::Locked { .. })
    }
}

fn hide_locked_things(
    upgrades: Res<GameUpgrades>,
    sources: Query<
        (&ItemsSource, &mut Visibility),
        (Added<ItemsSource>, Without<ProcessingStation>),
    >,
    stations: Query<(&ProcessingStation, &mut Visibility), Added<ProcessingStation>>,
) {
    for (source, mut visibility) in sources {
        if upgrades.item_is_locked(&source.item) {
            *visibility = Visibility::Hidden;
        }
    }
    for (station, mut visibility) in stations {
        if upgrades.station_is_locked(&station.kind) {
            *visibility = Visibility::Hidden;
        }
    }
}

fn check_upgrade_requirements(
    statistics: Res<Statistics>,
    mut upgrades: ResMut<GameUpgrades>,
    mut unlock_messages: MessageWriter<NewUnlock>,
    sources: Query<(&ItemsSource, &mut Visibility), Without<ProcessingStation>>,
    stations: Query<(&ProcessingStation, &mut Visibility)>,
) {
    for (item, state) in &mut upgrades.items {
        if let UpgradeState::Locked { requirements } = state
            && requirements.iter().all(|r| r.is_satisfied(&statistics))
        {
            *state = UpgradeState::Unlocked;
            unlock_messages.write(NewUnlock::Item(*item));
        }
    }

    for (station, state) in &mut upgrades.stations {
        if let UpgradeState::Locked { requirements } = state
            && requirements.iter().all(|r| r.is_satisfied(&statistics))
        {
            *state = UpgradeState::Unlocked;
            unlock_messages.write(NewUnlock::Station(*station));
        }
    }

    for (source, mut visibility) in sources {
        if *visibility == Visibility::Hidden && !upgrades.item_is_locked(&source.item) {
            *visibility = Visibility::Inherited;
        }
    }

    for (station, mut visibility) in stations {
        if *visibility == Visibility::Hidden && !upgrades.station_is_locked(&station.kind) {
            *visibility = Visibility::Inherited;
        }
    }
}
