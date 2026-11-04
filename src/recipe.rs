use crate::prelude::*;

#[derive(Debug)]
pub struct Recipe {
    customer_id: usize,
    ingredients: Vec<Ingredient>,
    stir_duration: i32,
}

#[derive(Debug)]
pub struct Ingredient {
    name: String,
    manual: String,
    item: Item,
}

#[rustfmt::skip]
pub fn manual_suffix(state: &ItemState, is_washed: &bool) -> String {
    match (is_washed, state) {
        (false, ItemState::Unprocessed) => format!("Just add it to the cauldron."),
        (false, ItemState::Sliced) => format!("Slice it and add it to the cauldron."),
        (false, ItemState::MortaredCoarse) => format!("Crush it and add it to the cauldron."),
        (false, ItemState::MortaredFine) => format!("Crush it well and add it to the cauldron."),
        (true, ItemState::Unprocessed) => format!("Wash it and add it to the cauldron."),
        (true, ItemState::Sliced) => format!("Wash it, slice it and add it to the cauldron."),
        (true, ItemState::MortaredCoarse) => format!("Wash it, crush it and add it to the cauldron."),
        (true, ItemState::MortaredFine) => format!("Wash it, crush it well and add it to the cauldron."),
    }
}

pub fn name_prefix(state: &ItemState, is_washed: &bool) -> String {
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

pub fn get_garlic_ingredients() -> Vec<Ingredient> {
    const IS_WASHED: [bool; 2] = [false, true];
    const STATES: [ItemState; 4] = [
        ItemState::Unprocessed,
        ItemState::Sliced,
        ItemState::MortaredCoarse,
        ItemState::MortaredFine,
    ];

    let mut ingredients = Vec::new();
    for is_washed in IS_WASHED.iter() {
        for state in STATES.iter() {
            let name = format!("{}Garlic", name_prefix(state, is_washed));
            ingredients.push(Ingredient {
                manual: format!("{}. {}", name, manual_suffix(state, is_washed)),
                item: Item::Garlic {
                    is_washed: *is_washed,
                    state: *state,
                },
                name,
            });
        }
    }

    ingredients
}

pub fn get_flower_ingredients() -> Vec<Ingredient> {
    const IS_WASHED: [bool; 1] = [false];
    const STATES: [ItemState; 3] = [
        ItemState::Unprocessed,
        ItemState::Sliced,
        ItemState::MortaredFine,
    ];

    let mut ingredients = Vec::new();
    for is_washed in IS_WASHED.iter() {
        for state in STATES.iter() {
            let name = format!("{}Flower", name_prefix(state, is_washed));
            ingredients.push(Ingredient {
                manual: format!("{}. {}", name, manual_suffix(state, is_washed)),
                item: Item::Garlic {
                    is_washed: *is_washed,
                    state: *state,
                },
                name,
            });
        }
    }
    ingredients
}

pub fn get_ingredients() -> Vec<Vec<Ingredient>> {
    let mut ingredients = Vec::new();
    ingredients.push(get_garlic_ingredients());
    ingredients.push(get_flower_ingredients());
    ingredients
}

pub const GREETINGS: [&'static str; 3] = [
    "Good Morning Greg. I've already prepared everything. Now get to work!",
    "Howdy My Friend. Feeling Good huh...? Yes? Get to work then!",
    "Hello Greg. I hope your morning was better than mine... No? Life goes on Anyways, just get to work!",
    // TODO
];

pub const CUSTOMERS: [&'static str; 4] = [
    "My Mother asked me whether I can do something for her Skin. Please be careful with this one!",
    "This one is for my cat too stop it from pooping everywhere!",
    "I think i've tried to make gold too often now. Anyways, be extra cautious!",
    "Note for Myself: Find out Albert's date of birth and make him a cake.",
    // TODO: more characters
];

pub const POSITIVE_RESULTS: [&'static str; 4] = [
    "She loves it. Thanks!",
    "I think it works, oh yeah... Oh no! My Bad. Well Done though.",
    "Not quite gold, but what I wanted. Well Done.",
    "Albert just invited me to his birthday! Perfect. Anyways, well done!",
];

pub const NEGATIVE_RESULTS: [&'static str; 4] = [
    "'Why is my skin yellow now?!?' 'I'm sorry Mother.' What did you do here?!? You need to improve your skills...",
    "My cat now has diarrhea. Thank you. Please do it more accurately next time...",
    "Not quite gold, but what I wanted. Well Done.",
    "Albert just invited me to his birthday! Perfect. Anyways, well done!",
];

pub const START_PREFIXES: [&'static str; 5] = [
    "Start with ",
    "Firstly add ",
    "First, ",
    "I think for this one start with ",
    "I'm not quite sure but try ",
];

pub const CONTINUE_PREFIXES: [&'static str; 6] = [
    "Then also add ",
    "Continue with some ",
    "And we also need some ",
    "I forgot what exactly but maybe some ",
    "We can also add some",
    "It would surely make sense to add some ",
];

pub const MORTAR_STEP_DESCRIPTIONS: [&'static str; 2] =
    ["short so it stays coarse", "thoroughly until it's fine"];

impl Recipe {
    pub fn manual(&self) -> String {
        let customer_text = CUSTOMERS[self.customer_id];
        let mut rng = rand::rng();

        let mut ingredient_texts = Vec::new();
        for (i, ingredient) in self.ingredients.iter().enumerate() {
            let prefix = if i == 0 {
                START_PREFIXES.choose(&mut rng).unwrap()
            } else {
                CONTINUE_PREFIXES.choose(&mut rng).unwrap()
            };

            let mut step_texts = Vec::new();
            for step in ingredient.processing_steps.iter() {
                let step_text = match step {
                    ProcessingStep::Mortar(Some(step)) => {
                        format!(
                            " We need to crush it in the mortar. Make sure to do it {}!",
                            MORTAR_STEP_DESCRIPTIONS[*step]
                        )
                    }
                    ProcessingStep::Mortar(_) => {
                        format!(
                            " We need to crush it in the mortar. Doesn't really matter how much."
                        )
                    }

                    ProcessingStep::Wash => {
                        format!(" Need to wash it until it's clean.")
                    }
                    ProcessingStep::Cut => {
                        format!(" Make sure to cut it ")
                    }
                };
                step_texts.push(step_text);
            }

            let step_texts = step_texts.join("");

            let ingredient_text = format!("{}{:?}.{} ", prefix, ingredient.raw_item, step_texts);

            ingredient_texts.push(ingredient_text);
        }

        let ingredient_texts = ingredient_texts.join("");

        let stir_text = format!("Make sure to stir for {} seconds!", self.stir_duration);

        format!(
            "{}\n\nIngredients:\n{}\n\n{}",
            customer_text, ingredient_texts, stir_text
        )
    }

    pub fn generate() -> Self {
        let mut available_items = vec![Item::Garlic, Item::Crystal, Item::Flowers, Item::Grass];

        let num_ingredients = rand::random_range(2..=available_items.len());

        let mut ingredients = Vec::new();
        for _ in 0..num_ingredients {
            let mut available_steps = vec![
                ProcessingStepKind::Mortar,
                ProcessingStepKind::Wash,
                ProcessingStepKind::Cut,
            ];

            let num_steps = rand::random_range(0..=available_steps.len());

            let item_idx = rand::random_range(0..available_items.len());
            let item = available_items.remove(item_idx);

            let mut steps = Vec::new();
            for _ in 0..num_steps {
                let step_idx = rand::random_range(0..available_steps.len());
                let step = match available_steps.remove(step_idx) {
                    ProcessingStepKind::Mortar => ProcessingStep::Mortar(None),
                    ProcessingStepKind::Wash => ProcessingStep::Wash,
                    ProcessingStepKind::Cut => ProcessingStep::Cut,
                };

                steps.push(step);
            }

            ingredients.push(Ingredient {
                raw_item: item,
                processing_steps: steps,
            });
        }

        let customer_id = rand::random_range(0..CUSTOMERS.len());
        let stir_duration = rand::random_range(10..=30);

        Self {
            customer_id,
            ingredients,
            stir_duration,
        }
    }
}
