use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use serde_json::Value;

use crate::card::{BasicLandType, CardData, CardType, Keyword, Supertype};
use crate::mana::ManaCost;

fn cache_path() -> PathBuf {
    let dir = dirs_or_home().join(".mtg-engine");
    fs::create_dir_all(&dir).ok();
    dir.join("card_cache.json")
}

fn dirs_or_home() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

fn load_cache() -> HashMap<String, Value> {
    let path = cache_path();
    if let Ok(data) = fs::read_to_string(&path) {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        HashMap::new()
    }
}

fn save_cache(cache: &HashMap<String, Value>) {
    let path = cache_path();
    if let Ok(data) = serde_json::to_string_pretty(cache) {
        fs::write(path, data).ok();
    }
}

pub fn fetch_card(name: &str) -> Result<CardData, String> {
    let mut cache = load_cache();
    let key = name.to_lowercase();

    let json = if let Some(cached) = cache.get(&key) {
        cached.clone()
    } else {
        println!("  Fetching from Scryfall: {}", name);
        thread::sleep(Duration::from_millis(80));

        let url = format!(
            "https://api.scryfall.com/cards/named?exact={}",
            urlencoded(name)
        );

        let resp = ureq::get(&url)
            .call()
            .map_err(|e| format!("HTTP error for '{}': {}", name, e))?;

        let json: Value = resp
            .into_json()
            .map_err(|e| format!("JSON parse error for '{}': {}", name, e))?;

        if json.get("object").and_then(|v| v.as_str()) == Some("error") {
            return Err(format!(
                "Scryfall error for '{}': {}",
                name,
                json.get("details").and_then(|v| v.as_str()).unwrap_or("unknown")
            ));
        }

        cache.insert(key, json.clone());
        save_cache(&cache);
        json
    };

    parse_scryfall_card(&json)
}

pub fn fetch_decklist(entries: &[(u8, &str)], status_fn: Option<&dyn Fn(&str)>) -> Vec<CardData> {
    let unique_names: Vec<&str> = {
        let mut seen = Vec::new();
        for (_, name) in entries {
            if !seen.contains(name) {
                seen.push(*name);
            }
        }
        seen
    };

    let mut card_map: HashMap<String, CardData> = HashMap::new();
    let total = unique_names.len();

    for (i, &name) in unique_names.iter().enumerate() {
        if let Some(f) = status_fn {
            f(&format!("Loading {}/{}: {}", i + 1, total, name));
        }
        match fetch_card(name) {
            Ok(card) => {
                card_map.insert(name.to_lowercase(), card);
            }
            Err(e) => {
                eprintln!("Warning: {}", e);
            }
        }
    }

    let mut deck = Vec::new();
    for &(count, name) in entries {
        if let Some(card) = card_map.get(&name.to_lowercase()) {
            for _ in 0..count {
                deck.push(card.clone());
            }
        }
    }

    deck
}

fn parse_scryfall_card(json: &Value) -> Result<CardData, String> {
    let name = json
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let mana_cost_str = json
        .get("mana_cost")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let cost = parse_mana_cost(mana_cost_str);

    let type_line = json
        .get("type_line")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let (supertypes, types, subtypes) = parse_type_line(type_line);

    let oracle_text = json
        .get("oracle_text")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let keywords = parse_keywords(json);

    let power = json
        .get("power")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<i32>().ok());

    let toughness = json
        .get("toughness")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<i32>().ok());

    let basic_land_types = parse_basic_land_types(&subtypes);

    Ok(CardData {
        name,
        cost,
        supertypes,
        types,
        subtypes: subtypes.iter().map(|s| s.to_string()).collect(),
        keywords,
        power,
        toughness,
        oracle_text,
        basic_land_types,
    })
}

fn parse_mana_cost(s: &str) -> ManaCost {
    let mut cost = ManaCost::new();
    let mut i = 0;
    let chars: Vec<char> = s.chars().collect();

    while i < chars.len() {
        if chars[i] == '{' {
            let end = chars.iter().skip(i).position(|&c| c == '}').unwrap_or(0) + i;
            let symbol = &s[i + 1..end];

            match symbol {
                "W" => cost.white += 1,
                "U" => cost.blue += 1,
                "B" => cost.black += 1,
                "R" => cost.red += 1,
                "G" => cost.green += 1,
                "C" => cost.colorless += 1,
                "X" => {} // X costs treated as 0
                n => {
                    if let Ok(num) = n.parse::<u8>() {
                        cost.generic += num;
                    }
                }
            }
            i = end + 1;
        } else {
            i += 1;
        }
    }

    cost
}

fn parse_type_line(type_line: &str) -> (Vec<Supertype>, Vec<CardType>, Vec<String>) {
    let parts: Vec<&str> = type_line.split(" — ").collect();
    let main_types_str = parts[0];
    let subtypes_str = if parts.len() > 1 { parts[1] } else { "" };

    let mut supertypes = Vec::new();
    let mut types = Vec::new();

    for word in main_types_str.split_whitespace() {
        match word {
            "Basic" => supertypes.push(Supertype::Basic),
            "Legendary" => supertypes.push(Supertype::Legendary),
            "Snow" => supertypes.push(Supertype::Snow),
            "Creature" => types.push(CardType::Creature),
            "Instant" => types.push(CardType::Instant),
            "Sorcery" => types.push(CardType::Sorcery),
            "Enchantment" => types.push(CardType::Enchantment),
            "Artifact" => types.push(CardType::Artifact),
            "Land" => types.push(CardType::Land),
            "Planeswalker" => types.push(CardType::Planeswalker),
            _ => {}
        }
    }

    let subtypes: Vec<String> = if subtypes_str.is_empty() {
        Vec::new()
    } else {
        subtypes_str
            .split_whitespace()
            .map(|s| s.to_string())
            .collect()
    };

    (supertypes, types, subtypes)
}

fn parse_keywords(json: &Value) -> Vec<Keyword> {
    let mut keywords = Vec::new();

    if let Some(arr) = json.get("keywords").and_then(|v| v.as_array()) {
        for kw in arr {
            if let Some(s) = kw.as_str() {
                match s {
                    "Flying" => keywords.push(Keyword::Flying),
                    "First strike" => keywords.push(Keyword::FirstStrike),
                    "Double strike" => keywords.push(Keyword::DoubleStrike),
                    "Deathtouch" => keywords.push(Keyword::Deathtouch),
                    "Lifelink" => keywords.push(Keyword::Lifelink),
                    "Trample" => keywords.push(Keyword::Trample),
                    "Vigilance" => keywords.push(Keyword::Vigilance),
                    "Reach" => keywords.push(Keyword::Reach),
                    "Haste" => keywords.push(Keyword::Haste),
                    "Hexproof" => keywords.push(Keyword::Hexproof),
                    "Indestructible" => keywords.push(Keyword::Indestructible),
                    "Menace" => keywords.push(Keyword::Menace),
                    "Flash" => keywords.push(Keyword::Flash),
                    "Defender" => keywords.push(Keyword::Defender),
                    _ => {}
                }
            }
        }
    }

    keywords
}

fn parse_basic_land_types(subtypes: &[String]) -> Vec<BasicLandType> {
    let mut land_types = Vec::new();
    for st in subtypes {
        match st.as_str() {
            "Plains" => land_types.push(BasicLandType::Plains),
            "Island" => land_types.push(BasicLandType::Island),
            "Swamp" => land_types.push(BasicLandType::Swamp),
            "Mountain" => land_types.push(BasicLandType::Mountain),
            "Forest" => land_types.push(BasicLandType::Forest),
            _ => {}
        }
    }
    land_types
}

fn urlencoded(s: &str) -> String {
    s.replace(' ', "+")
        .replace('\'', "%27")
        .replace(',', "%2C")
}
