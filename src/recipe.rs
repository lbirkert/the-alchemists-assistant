use crate::prelude::*;

#[derive(Reflect, Clone, Copy, Debug)]
pub enum ProcessingStep {
    MortarAndPestle(Option<usize>),
}

pub enum ProcessingStepKind {
    MortarAndPestle,
}

#[derive(Component, Reflect, Clone, Debug)]
#[reflect(Component)]
pub struct Ingredient {
    raw_item: Item,
    processing_steps: Vec<ProcessingStep>,
}

#[derive(Debug)]
pub struct Recipe {
    customer_id: usize,
    ingredients: Vec<Ingredient>,
    stir_duration: i32,
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
    "We can also add some ",
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
                    ProcessingStep::MortarAndPestle(Some(step)) => {
                        format!(
                            " We need to crush it in the mortar. Make sure to do it {}!",
                            MORTAR_STEP_DESCRIPTIONS[*step]
                        )
                    }
                    ProcessingStep::MortarAndPestle(_) => {
                        format!(
                            " We need to crush it in the mortar. Doesn't really matter how much."
                        )
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
            let mut available_steps = vec![ProcessingStepKind::MortarAndPestle];

            let num_steps = rand::random_range(0..=available_steps.len());

            let item_idx = rand::random_range(0..available_items.len());
            let item = available_items.remove(item_idx);

            let mut steps = Vec::new();
            for _ in 0..num_steps {
                let step_idx = rand::random_range(0..available_steps.len());
                let step = match available_steps.remove(step_idx) {
                    ProcessingStepKind::MortarAndPestle => ProcessingStep::MortarAndPestle(None),
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
