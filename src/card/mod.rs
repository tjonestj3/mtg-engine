use serde::{Deserialize, Serialize};
use std::fmt;

use crate::mana::{Color, ManaCost};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CardType {
    Creature,
    Instant,
    Sorcery,
    Enchantment,
    Artifact,
    Land,
    Planeswalker,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Supertype {
    Basic,
    Legendary,
    Snow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Keyword {
    Flying,
    FirstStrike,
    DoubleStrike,
    Deathtouch,
    Lifelink,
    Trample,
    Vigilance,
    Reach,
    Haste,
    Hexproof,
    Indestructible,
    Menace,
    Flash,
    Defender,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BasicLandType {
    Plains,
    Island,
    Swamp,
    Mountain,
    Forest,
}

impl BasicLandType {
    pub fn produces(&self) -> Color {
        match self {
            BasicLandType::Plains => Color::White,
            BasicLandType::Island => Color::Blue,
            BasicLandType::Swamp => Color::Black,
            BasicLandType::Mountain => Color::Red,
            BasicLandType::Forest => Color::Green,
        }
    }
}

pub type CardId = u64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardData {
    pub name: String,
    pub cost: ManaCost,
    pub supertypes: Vec<Supertype>,
    pub types: Vec<CardType>,
    pub subtypes: Vec<String>,
    pub keywords: Vec<Keyword>,
    pub power: Option<i32>,
    pub toughness: Option<i32>,
    pub oracle_text: String,
    pub basic_land_types: Vec<BasicLandType>,
    #[serde(default)]
    pub image_file: Option<String>,
}

impl CardData {
    pub fn is_type(&self, card_type: CardType) -> bool {
        self.types.contains(&card_type)
    }

    pub fn is_creature(&self) -> bool {
        self.is_type(CardType::Creature)
    }

    pub fn is_land(&self) -> bool {
        self.is_type(CardType::Land)
    }

    pub fn is_instant(&self) -> bool {
        self.is_type(CardType::Instant)
    }

    pub fn has_keyword(&self, keyword: Keyword) -> bool {
        self.keywords.contains(&keyword)
    }

    pub fn has_flash(&self) -> bool {
        self.has_keyword(Keyword::Flash)
    }

    pub fn color_identity(&self) -> Vec<Color> {
        self.cost.colors()
    }
}

impl fmt::Display for CardData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)?;
        if self.cost.converted_mana_cost() > 0 {
            write!(f, " {}", self.cost)?;
        }
        if let (Some(p), Some(t)) = (self.power, self.toughness) {
            write!(f, " {}/{}", p, t)?;
        }
        Ok(())
    }
}
