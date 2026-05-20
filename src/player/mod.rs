use serde::{Deserialize, Serialize};
use std::fmt;

use crate::mana::ManaPool;
use crate::zone::ZoneManager;

pub type PlayerId = u8;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: PlayerId,
    pub name: String,
    pub life: i32,
    pub mana_pool: ManaPool,
    pub zones: ZoneManager,
    pub land_plays_remaining: u8,
    pub has_drawn_for_turn: bool,
}

impl Player {
    pub fn new(id: PlayerId, name: String) -> Self {
        Self {
            id,
            name,
            life: 20,
            mana_pool: ManaPool::new(),
            zones: ZoneManager::new(),
            land_plays_remaining: 1,
            has_drawn_for_turn: false,
        }
    }

    pub fn is_alive(&self) -> bool {
        self.life > 0 && !self.zones.library.is_empty()
    }

    pub fn deal_damage(&mut self, amount: i32) {
        self.life -= amount;
    }

    pub fn gain_life(&mut self, amount: i32) {
        self.life += amount;
    }

    pub fn can_play_land(&self) -> bool {
        self.land_plays_remaining > 0
    }

    pub fn reset_for_turn(&mut self) {
        self.land_plays_remaining = 1;
        self.has_drawn_for_turn = false;
    }

    pub fn hand_size(&self) -> usize {
        self.zones.hand.len()
    }

    pub fn library_size(&self) -> usize {
        self.zones.library.len()
    }
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} — Life: {} | Hand: {} | Library: {} | Mana: {}",
            self.name,
            self.life,
            self.hand_size(),
            self.library_size(),
            self.mana_pool,
        )
    }
}
