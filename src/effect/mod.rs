use serde::{Deserialize, Serialize};

use crate::card::{CardId, Keyword};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermanentState {
    pub card_id: CardId,
    pub tapped: bool,
    pub summoning_sick: bool,
    pub damage_marked: i32,
    pub counters: Counters,
    pub attached_to: Option<CardId>,
    pub extra_keywords: Vec<Keyword>,
    pub power_modifier: i32,
    pub toughness_modifier: i32,
}

impl PermanentState {
    pub fn new(card_id: CardId) -> Self {
        Self {
            card_id,
            tapped: false,
            summoning_sick: true,
            damage_marked: 0,
            counters: Counters::default(),
            attached_to: None,
            extra_keywords: Vec::new(),
            power_modifier: 0,
            toughness_modifier: 0,
        }
    }

    pub fn tap(&mut self) -> bool {
        if self.tapped {
            return false;
        }
        self.tapped = true;
        true
    }

    pub fn untap(&mut self) {
        self.tapped = false;
    }

    pub fn can_attack(&self) -> bool {
        !self.tapped && !self.summoning_sick
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Counters {
    pub plus_one: i32,
    pub minus_one: i32,
    pub loyalty: i32,
}
