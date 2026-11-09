use crate::{
    item::ItemSource,
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

    pub fn fill_potion(&mut self) {
        self.potions_brewed += 1;
    }
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct GameUpgrades {
    items: HashMap<ItemKind, UpgradeState>,
    stations: HashMap<Station, UpgradeState>,
}

#[derive(Reflect)]
enum UpgradeState {
    UnlockedByDefault,
    Locked {
        requirements: Vec<UnlockCondition>,
        is_implemented: bool,
    },
    Unlocked,
}

impl UpgradeState {
    fn locked(requirements: impl IntoIterator<Item = UnlockCondition>) -> Self {
        Self::Locked {
            requirements: requirements.into_iter().collect(),
            is_implemented: true,
        }
    }

    fn unimplemented() -> Self {
        Self::Locked {
            requirements: vec![],
            is_implemented: false,
        }
    }
}

#[derive(Message, Clone, Copy, Debug)]
pub enum NewUnlock {
    Item(ItemKind),
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
        let items = ItemKind::iter()
            .map(|item| {
                (
                    item,
                    match item {
                        ItemKind::Garlic => UpgradeState::UnlockedByDefault,
                        ItemKind::Flowers => UpgradeState::UnlockedByDefault,
                        ItemKind::Potion => UpgradeState::UnlockedByDefault,
                        ItemKind::Grass => UpgradeState::locked([UnlockCondition::item(
                            Item {
                                kind: ItemKind::Flowers,
                                state: ItemState::Grinded,
                                is_washed: false,
                                potion: None,
                            },
                            3,
                        )]),
                        ItemKind::Crystal => {
                            UpgradeState::locked([UnlockCondition::TotalItemsProcessed(10)])
                        }
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
                        Station::Mortar => {
                            UpgradeState::locked([UnlockCondition::TotalItemsProcessed(5)])
                        }
                        Station::WaterTap => {
                            UpgradeState::locked([UnlockCondition::TotalPotionsBrewed(5)])
                        }
                    },
                )
            })
            .collect();
        Self { items, stations }
    }
}

impl GameUpgrades {
    pub fn item_is_locked(&self, item: &Item) -> bool {
        let Item {
            kind,
            state,
            is_washed,
            potion,
        } = item;
        if potion.is_some() {
            error!("Checking for locked state of potion was not intended");
            return true;
        }

        let kind_is_locked = matches!(self.items[kind], UpgradeState::Locked { .. });

        let mut station_is_locked = false;
        if *is_washed && self.station_is_locked(&Station::WaterTap) {
            station_is_locked = true;
        }
        if let Some(station) = state.needs_station()
            && self.station_is_locked(&station)
        {
            station_is_locked = true;
        }

        kind_is_locked || station_is_locked
    }

    fn item_kind_is_locked(&self, item_kind: &ItemKind) -> bool {
        matches!(self.items[item_kind], UpgradeState::Locked { .. })
    }

    fn station_is_locked(&self, station: &Station) -> bool {
        matches!(self.stations[station], UpgradeState::Locked { .. })
    }
}

fn hide_locked_things(
    upgrades: Res<GameUpgrades>,
    sources: Query<(&ItemSource, &mut Visibility), (Added<ItemSource>, Without<ProcessingStation>)>,
    stations: Query<(&ProcessingStation, &mut Visibility), Added<ProcessingStation>>,
) {
    for (source, mut visibility) in sources {
        if upgrades.item_kind_is_locked(&source.item.kind) {
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
    mut commands: Commands,
    mut upgrades: ResMut<GameUpgrades>,
    mut unlock_messages: MessageWriter<NewUnlock>,
    sources: Query<(&ItemSource, &mut Visibility), Without<ProcessingStation>>,
    stations: Query<(&ProcessingStation, &mut Visibility)>,
) {
    for (item, state) in &mut upgrades.items {
        if let UpgradeState::Locked {
            requirements,
            is_implemented: true,
        } = state
            && requirements.iter().all(|r| r.is_satisfied(&statistics))
        {
            *state = UpgradeState::Unlocked;
            commands.spawn_scene(audio_scene_not_spatial(&["audio/bottle_opening.ogg"]));
            unlock_messages.write(NewUnlock::Item(*item));
        }
    }

    for (station, state) in &mut upgrades.stations {
        if let UpgradeState::Locked {
            requirements,
            is_implemented: true,
        } = state
            && requirements.iter().all(|r| r.is_satisfied(&statistics))
        {
            *state = UpgradeState::Unlocked;
            commands.spawn_scene(audio_scene_not_spatial(&["audio/bottle_opening.ogg"]));
            unlock_messages.write(NewUnlock::Station(*station));
        }
    }

    for (source, mut visibility) in sources {
        if *visibility == Visibility::Hidden && !upgrades.item_kind_is_locked(&source.item.kind) {
            *visibility = Visibility::Inherited;
        }
    }

    for (station, mut visibility) in stations {
        if *visibility == Visibility::Hidden && !upgrades.station_is_locked(&station.kind) {
            *visibility = Visibility::Inherited;
        }
    }
}
