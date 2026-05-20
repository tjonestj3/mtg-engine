use serde::{Deserialize, Serialize};

use crate::card::CardId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Zone {
    Library,
    Hand,
    Battlefield,
    Graveyard,
    Exile,
    Stack,
    Command,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneManager {
    pub library: Vec<CardId>,
    pub hand: Vec<CardId>,
    pub battlefield: Vec<CardId>,
    pub graveyard: Vec<CardId>,
    pub exile: Vec<CardId>,
}

impl ZoneManager {
    pub fn new() -> Self {
        Self {
            library: Vec::new(),
            hand: Vec::new(),
            battlefield: Vec::new(),
            graveyard: Vec::new(),
            exile: Vec::new(),
        }
    }

    pub fn get_zone(&self, zone: Zone) -> &[CardId] {
        match zone {
            Zone::Library => &self.library,
            Zone::Hand => &self.hand,
            Zone::Battlefield => &self.battlefield,
            Zone::Graveyard => &self.graveyard,
            Zone::Exile => &self.exile,
            Zone::Stack | Zone::Command => &[],
        }
    }

    pub fn move_card(&mut self, card_id: CardId, from: Zone, to: Zone) -> bool {
        let removed = self.remove_from(card_id, from);
        if removed {
            self.add_to(card_id, to);
        }
        removed
    }

    fn remove_from(&mut self, card_id: CardId, zone: Zone) -> bool {
        let vec = match zone {
            Zone::Library => &mut self.library,
            Zone::Hand => &mut self.hand,
            Zone::Battlefield => &mut self.battlefield,
            Zone::Graveyard => &mut self.graveyard,
            Zone::Exile => &mut self.exile,
            Zone::Stack | Zone::Command => return false,
        };
        if let Some(pos) = vec.iter().position(|&id| id == card_id) {
            vec.remove(pos);
            true
        } else {
            false
        }
    }

    fn add_to(&mut self, card_id: CardId, zone: Zone) {
        match zone {
            Zone::Library => self.library.push(card_id),
            Zone::Hand => self.hand.push(card_id),
            Zone::Battlefield => self.battlefield.push(card_id),
            Zone::Graveyard => self.graveyard.push(card_id),
            Zone::Exile => self.exile.push(card_id),
            Zone::Stack | Zone::Command => {}
        }
    }

    pub fn draw_from_library(&mut self) -> Option<CardId> {
        if self.library.is_empty() {
            return None;
        }
        let card_id = self.library.remove(0);
        self.hand.push(card_id);
        Some(card_id)
    }

    pub fn shuffle_library(&mut self) {
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        self.library.shuffle(&mut rng);
    }
}

impl Default for ZoneManager {
    fn default() -> Self {
        Self::new()
    }
}
