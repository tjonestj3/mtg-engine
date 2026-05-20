use serde::{Deserialize, Serialize};

use crate::card::CardId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackerInfo {
    pub card_id: CardId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockerAssignment {
    pub blocker_id: CardId,
    pub blocking: CardId,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CombatState {
    pub attackers: Vec<AttackerInfo>,
    pub blockers: Vec<BlockerAssignment>,
    pub damage_assigned: bool,
}

impl CombatState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn declare_attacker(&mut self, card_id: CardId) {
        self.attackers.push(AttackerInfo { card_id });
    }

    pub fn declare_blocker(&mut self, blocker_id: CardId, blocking: CardId) {
        self.blockers.push(BlockerAssignment { blocker_id, blocking });
    }

    pub fn get_blockers_for(&self, attacker_id: CardId) -> Vec<CardId> {
        self.blockers
            .iter()
            .filter(|b| b.blocking == attacker_id)
            .map(|b| b.blocker_id)
            .collect()
    }

    pub fn is_blocked(&self, attacker_id: CardId) -> bool {
        self.blockers.iter().any(|b| b.blocking == attacker_id)
    }

    pub fn clear(&mut self) {
        self.attackers.clear();
        self.blockers.clear();
        self.damage_assigned = false;
    }
}
