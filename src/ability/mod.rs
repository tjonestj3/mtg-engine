use serde::{Deserialize, Serialize};

use crate::card::CardId;
use crate::player::PlayerId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AbilityKind {
    Activated,
    Triggered,
    Static,
    Mana,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TargetKind {
    Player(PlayerId),
    Permanent(CardId),
    Card { zone: crate::zone::Zone, card_id: CardId },
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Effect {
    DealDamage { target: TargetKind, amount: i32 },
    GainLife { player: PlayerId, amount: i32 },
    DrawCards { player: PlayerId, count: u8 },
    DestroyPermanent { target: CardId },
    AddMana { color: crate::mana::Color, amount: u8 },
    AddColorlessMana { amount: u8 },
    Untap { target: CardId },
    Tap { target: CardId },
    ReturnToHand { target: CardId },
    SearchLibrary { player: PlayerId, card_type: Option<crate::card::CardType> },
    CreateToken { power: i32, toughness: i32, types: Vec<crate::card::CardType> },
    Counter { target_on_stack: usize },
}
