use crate::prelude::*;

// a potion crafted by a user that might have a sticky note attached
#[derive(Reflect, Clone, Debug)]
pub struct Potion {
    items: Vec<Item>,
    // TODO: type: round, square, etc...
    sticky_note: Option<StickyNote>,
}

impl Potion {
    pub fn can_validate(&self) -> bool {
        self.sticky_note
            .as_ref()
            .is_some_and(|s| s.recipe.is_some())
    }

    pub fn validate(&self) -> Option<bool> {
        if let Some(StickyNote {
            recipe: Some(ref recipe),
            ..
        }) = self.sticky_note
        {
            Some(self.items == recipe.items())
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Reflect, PartialEq, Eq)]
pub struct Recipe {
    pub task_description: String,
    pub positive_result: String,
    pub negative_result: String,
    pub ingredients: Vec<Ingredient>,
    pub stir_duration: i32, // do we need this?
}

#[derive(Debug, Clone, Reflect, PartialEq, Eq)]
pub struct Ingredient {
    steps_description: String,
    item: Item,
}

// for faervans PR
pub struct Upgrades;

enum NameGender {
    Male,
    Female,
}

pub const GREETINGS: [&'static str; 3] = [
    "Good Morning Greg. I've already prepared everything. Now get to work!",
    "Howdy My Friend. Feeling Good huh...? Yes? Get to work then!",
    "Hello Greg. I hope your morning was better than mine... No? Life goes on Anyways, just get to work!",
    // TODO
];

const NAMES: [(NameGender, &'static str); 10] = [
    (NameGender::Male, "Nate"),
    (NameGender::Male, "Albert"),
    (NameGender::Male, "Vincent"),
    (NameGender::Male, "Walter"),
    (NameGender::Male, "Bruno"),
    (NameGender::Female, "Anna"),
    (NameGender::Female, "Natasha"),
    (NameGender::Female, "Louise"),
    (NameGender::Female, "Sofia"),
    (NameGender::Female, "Lex"),
];

const PROBLEMS: [&'static str; 10] = [
    "{name} asked me to cure {article_possessive} cancer. Let's give it a try!",
    "Yesterday I met {name} and {article_subject} asked me how to make {article_possessive} room plant grow bigger. Here's the recipe:",
    "Have I told you about {name} yet? Anyways, {article_subject} needs something against {article_possessive} warts.",
    "I think i've tried making gold enough. Anyways, try this:",
    "Plankton asked me to recreate the Krabby Patty Secret Formula. Try this please:",
    "I've got a pimple. Please stir this one up for me:",
    "Donald asked me to cook something against those Democrats. Here's the recipe:",
    "Child labor is awful! Anyways, can you cook this one up for me to extract more cobalt from the mines in africa?",
    "{name}'s dog is barking all the time. This one's for the little rat:",
    "Our plants look kinda dull. Cook this one up for me:",
];

const POSITIVE_RESULTS: [&'static str; 10] = [
    "{article_subject} is cancer-free! Wait, that's not how medicine works... Anyways, well done!",
    "The plant is HUGE! {article_subject} might need a bigger room now. Well done!",
    "{article_possessive} warts are gone! {article_subject} says you're a miracle worker. Thank you!",
    "This might actually make us rich! Investors are already calling. Well done!",
    "Plankton says it tastes exactly like the original. He's suspiciously happy. Good job!",
    "The pimple is gone! My face has never looked this good. Well done!",
    "Donald loves it! He says it's exactly what America needed. Well done!",
    "The cobalt extraction rate has doubled! The workers are... surprisingly productive. Well done!",
    "The dog hasn't barked in hours. The neighbors (and me) are finally at peace. Well done!",
    "The plants look amazing! My garden is basically a jungle now. Well done!",
];

const NEGATIVE_RESULTS: [&'static str; 10] = [
    "I don't think this bump's normal! What did you put in there?! Maybe leave medicine to the professionals...",
    "The plant is dead. {article_subject} specifically asked you NOT to kill it. What did you do?!",
    "{article_possessive} warts have quadrupled! {article_subject} is starting to look like a toad. Please improve your skills...",
    "We're somehow even poorer now. Maybe Mom was right when she told me to get a *real* job...",
    "I don't think Mr. Crabs approves this. Try again next time...",
    "My pimple has grown a friend. They're practically a family now. What do I do now?!",
    "Donald is furious! The Democrats just multiplied. Quite an achievement...",
    "The cobalt production has stopped entirely due to hazardous substances. Maybe need try that again...",
    "The dog is now barking twice as loud. The neighbors are threatening to call the cops on me...",
    "The plants look even worse! One of them actually turned brown while I was watching. You need to improve...",
];

const START_PREFIXES: [&'static str; 5] = [
    "Start with some ",
    "Begin with some ",
    "First, add some ",
    "For this one start with ",
    "Try to start with some ",
];

const CONTINUE_PREFIXES: [&'static str; 6] = [
    "Also add some ",
    "Continue with some ",
    "We also need some ",
    "Maybe add some ",
    "We can also add some ",
    "Add some ",
];

impl Recipe {
    #[rustfmt::skip]
    fn steps_description(state: &ItemState, is_washed: &bool) -> String {
    match (is_washed, state) {
        (false, ItemState::Unprocessed) => format!("Just add to the cauldron."),
        (false, ItemState::Sliced) => format!("Slice and add to the cauldron."),
        (false, ItemState::Grinded) => format!("Grind and add to the cauldron."),
        (false, ItemState::FineGrinded) => format!("Grind fine! and add to the cauldron."),
        (true, ItemState::Unprocessed) => format!("Wash and add to the cauldron."),
        (true, ItemState::Sliced) => format!("Wash, slice and add to the cauldron."),
        (true, ItemState::Grinded) => format!("Wash, grind and add to the cauldron."),
        (true, ItemState::FineGrinded) => format!("Wash, grind fine! and add to the cauldron."),
    }
}

    fn get_garlic_ingredients(upgrades: &Upgrades) -> Vec<Ingredient> {
        const IS_WASHED: [bool; 2] = [false, true];
        const STATES: [ItemState; 4] = [
            ItemState::Unprocessed,
            ItemState::Sliced,
            ItemState::Grinded,
            ItemState::FineGrinded,
        ];

        let mut ingredients = Vec::new();
        for is_washed in IS_WASHED.iter() {
            for state in STATES.iter() {
                // TODO: check upgrades for valid ingredients
                ingredients.push(Ingredient {
                    steps_description: Self::steps_description(state, is_washed),
                    item: Item::Garlic {
                        is_washed: *is_washed,
                        state: *state,
                    },
                });
            }
        }

        ingredients
    }

    fn get_flowers_ingredients(upgrades: &Upgrades) -> Vec<Ingredient> {
        const IS_WASHED: [bool; 1] = [false];
        const STATES: [ItemState; 3] = [
            ItemState::Unprocessed,
            ItemState::Sliced,
            ItemState::FineGrinded,
        ];

        let mut ingredients = Vec::new();
        for is_washed in IS_WASHED.iter() {
            for state in STATES.iter() {
                // TODO: check upgrades for valid ingredients
                ingredients.push(Ingredient {
                    steps_description: Self::steps_description(state, is_washed),
                    item: Item::Flowers {
                        is_washed: *is_washed,
                        state: *state,
                    },
                });
            }
        }

        ingredients
    }

    fn get_ingredients(upgrades: &Upgrades) -> Vec<Vec<Ingredient>> {
        let mut ingredients = Vec::new();
        ingredients.push(Self::get_garlic_ingredients(&upgrades));
        ingredients.push(Self::get_flowers_ingredients(&upgrades));
        ingredients
    }

    pub fn items(&self) -> Vec<Item> {
        self.ingredients
            .iter()
            .map(|i| i.item.clone())
            .collect::<Vec<_>>()
    }

    pub fn generate() -> Self {
        let mut rng = rand::rng();
        let mut available_ingredients = Self::get_ingredients(&Upgrades);
        let num_ingredients = rand::random_range(2..=available_ingredients.len());

        let mut ingredients = Vec::new();
        let mut ingredient_texts = Vec::new();
        for ingredient_no in 0..num_ingredients {
            let index = rand::random_range(0..available_ingredients.len());
            let mut sub_ingredients = available_ingredients.remove(index);
            let index = rand::random_range(0..sub_ingredients.len());
            let ingredient = sub_ingredients.remove(index);

            let prefix = if ingredient_no == 0 {
                START_PREFIXES.choose(&mut rng).unwrap()
            } else {
                CONTINUE_PREFIXES.choose(&mut rng).unwrap()
            };

            let ingredient_text = format!(
                "{}{}. {}",
                prefix,
                ingredient.item.short(),
                ingredient.steps_description
            );

            ingredient_texts.push(ingredient_text);
            ingredients.push(ingredient);
        }

        let ingredient_texts = ingredient_texts.join("\n");
        let stir_duration = rand::random_range(10..=30);

        let stir_text = format!("Stir for {} seconds and fill in a flask!", stir_duration);

        let name_id = rand::random_range(0..NAMES.len());
        let (ref gender, name) = NAMES[name_id];
        let (article_subject, article_object, article_possesive) = match gender {
            NameGender::Female => ("She", "Her", "Her"),
            NameGender::Male => ("He", "Him", "His"),
        };

        let problem_id = rand::random_range(0..PROBLEMS.len());
        let problem_text = PROBLEMS[problem_id]
            .replace("{name}", name)
            .replace("{article_subject}", article_subject)
            .replace("{article_object}", article_object)
            .replace("{article_possessive}", article_possesive);

        let positive_result = POSITIVE_RESULTS[problem_id]
            .replace("{name}", name)
            .replace("{article_subject}", article_subject)
            .replace("{article_object}", article_object)
            .replace("{article_possessive}", article_possesive);

        let negative_result = NEGATIVE_RESULTS[problem_id]
            .replace("{name}", name)
            .replace("{article_subject}", article_subject)
            .replace("{article_object}", article_object)
            .replace("{article_possessive}", article_possesive);

        let task_description = format!(
            "{}\n\nIngredients:\n{}\n\n{}",
            problem_text, ingredient_texts, stir_text
        );

        // generate

        Self {
            task_description,
            positive_result,
            negative_result,
            ingredients,
            stir_duration,
        }
    }
}
