pub mod decklists;
pub mod scryfall;

use crate::card::{BasicLandType, CardData, CardType, Keyword, Supertype};
use crate::mana::ManaCost;

pub fn basic_land(name: &str, land_type: BasicLandType) -> CardData {
    CardData {
        name: name.to_string(),
        cost: ManaCost::new(),
        supertypes: vec![Supertype::Basic],
        types: vec![CardType::Land],
        subtypes: vec![format!("{:?}", land_type)],
        keywords: vec![],
        power: None,
        toughness: None,
        oracle_text: format!("({{T}}: Add {{{}}})", land_type.produces()),
        basic_land_types: vec![land_type],
        image_file: None,
    }
}

pub fn vanilla_creature(name: &str, cost: ManaCost, power: i32, toughness: i32) -> CardData {
    CardData {
        name: name.to_string(),
        cost,
        supertypes: vec![],
        types: vec![CardType::Creature],
        subtypes: vec![],
        keywords: vec![],
        power: Some(power),
        toughness: Some(toughness),
        oracle_text: String::new(),
        basic_land_types: vec![],
        image_file: None,
    }
}

pub fn creature_with_keywords(
    name: &str,
    cost: ManaCost,
    power: i32,
    toughness: i32,
    keywords: Vec<Keyword>,
) -> CardData {
    CardData {
        name: name.to_string(),
        cost,
        supertypes: vec![],
        types: vec![CardType::Creature],
        subtypes: vec![],
        keywords,
        power: Some(power),
        toughness: Some(toughness),
        oracle_text: String::new(),
        basic_land_types: vec![],
        image_file: None,
    }
}

pub fn sample_decklist() -> Vec<CardData> {
    let mut deck = Vec::new();
    for _ in 0..8 { deck.push(basic_land("Forest", BasicLandType::Forest)); }
    for _ in 0..8 { deck.push(basic_land("Mountain", BasicLandType::Mountain)); }
    for _ in 0..4 { deck.push(vanilla_creature("Grizzly Bears", ManaCost { generic: 1, green: 1, ..ManaCost::new() }, 2, 2)); }
    for _ in 0..4 { deck.push(vanilla_creature("Hill Giant", ManaCost { generic: 3, red: 1, ..ManaCost::new() }, 3, 3)); }
    for _ in 0..4 { deck.push(creature_with_keywords("Llanowar Elves", ManaCost { green: 1, ..ManaCost::new() }, 1, 1, vec![])); }
    for _ in 0..4 { deck.push(creature_with_keywords("Serra Angel", ManaCost { generic: 3, white: 2, ..ManaCost::new() }, 4, 4, vec![Keyword::Flying, Keyword::Vigilance])); }
    for _ in 0..4 { deck.push(vanilla_creature("Goblin Piker", ManaCost { generic: 1, red: 1, ..ManaCost::new() }, 2, 1)); }
    for _ in 0..4 { deck.push(creature_with_keywords("Giant Spider", ManaCost { generic: 3, green: 1, ..ManaCost::new() }, 2, 4, vec![Keyword::Reach])); }
    deck
}
