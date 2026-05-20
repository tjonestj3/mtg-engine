use std::collections::HashMap;
use std::fmt;

use crate::card::{CardData, CardId};
use crate::combat::CombatState;
use crate::effect::PermanentState;
use crate::player::{Player, PlayerId};
use crate::stack::GameStack;

use super::turns::TurnState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameStatus {
    InProgress,
    Won(PlayerId),
    Draw,
}

pub struct GameState {
    pub players: Vec<Player>,
    pub turn: TurnState,
    pub stack: GameStack,
    pub combat: CombatState,
    pub permanents: HashMap<CardId, PermanentState>,
    pub card_registry: HashMap<CardId, CardData>,
    pub next_card_id: CardId,
    pub status: GameStatus,
}

impl GameState {
    pub fn new(player_names: Vec<String>) -> Self {
        let players = player_names
            .into_iter()
            .enumerate()
            .map(|(i, name)| Player::new(i as PlayerId, name))
            .collect();

        Self {
            players,
            turn: TurnState::new(),
            stack: GameStack::new(),
            combat: CombatState::new(),
            permanents: HashMap::new(),
            card_registry: HashMap::new(),
            next_card_id: 1,
            status: GameStatus::InProgress,
        }
    }

    pub fn register_card(&mut self, data: CardData) -> CardId {
        let id = self.next_card_id;
        self.next_card_id += 1;
        self.card_registry.insert(id, data);
        id
    }

    pub fn get_card(&self, id: CardId) -> Option<&CardData> {
        self.card_registry.get(&id)
    }

    pub fn get_permanent(&self, id: CardId) -> Option<&PermanentState> {
        self.permanents.get(&id)
    }

    pub fn get_permanent_mut(&mut self, id: CardId) -> Option<&mut PermanentState> {
        self.permanents.get_mut(&id)
    }

    pub fn active_player(&self) -> &Player {
        &self.players[self.turn.active_player as usize]
    }

    pub fn active_player_mut(&mut self) -> &mut Player {
        &mut self.players[self.turn.active_player as usize]
    }

    pub fn player(&self, id: PlayerId) -> &Player {
        &self.players[id as usize]
    }

    pub fn player_mut(&mut self, id: PlayerId) -> &mut Player {
        &mut self.players[id as usize]
    }

    pub fn check_state_based_actions(&mut self) {
        let mut died = Vec::new();

        for player in &self.players {
            if player.life <= 0 {
                died.push(player.id);
            }
        }

        let mut to_graveyard = Vec::new();
        for (card_id, perm) in &self.permanents {
            if let Some(card) = self.card_registry.get(card_id) {
                if card.is_creature() {
                    if let Some(toughness) = card.toughness {
                        let effective_toughness =
                            toughness + perm.toughness_modifier + perm.counters.plus_one - perm.counters.minus_one;
                        if effective_toughness <= 0 || perm.damage_marked >= effective_toughness {
                            to_graveyard.push(*card_id);
                        }
                    }
                }
            }
        }

        for card_id in to_graveyard {
            self.permanents.remove(&card_id);
            for player in &mut self.players {
                if player.zones.battlefield.contains(&card_id) {
                    player
                        .zones
                        .move_card(card_id, crate::zone::Zone::Battlefield, crate::zone::Zone::Graveyard);
                    break;
                }
            }
        }

        if !died.is_empty() {
            if died.len() >= self.players.len() {
                self.status = GameStatus::Draw;
            } else {
                let winner = self
                    .players
                    .iter()
                    .find(|p| !died.contains(&p.id))
                    .map(|p| p.id)
                    .unwrap_or(0);
                self.status = GameStatus::Won(winner);
            }
        }
    }
}

impl fmt::Display for GameState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "=== Turn {} — {:?} ===", self.turn.turn_number, self.turn.step)?;
        for player in &self.players {
            let active = if player.id == self.turn.active_player { " (active)" } else { "" };
            writeln!(f, "  {}{}", player, active)?;

            let battlefield_names: Vec<String> = player
                .zones
                .battlefield
                .iter()
                .filter_map(|id| self.card_registry.get(id))
                .map(|c| c.name.clone())
                .collect();
            if !battlefield_names.is_empty() {
                writeln!(f, "    Battlefield: {}", battlefield_names.join(", "))?;
            }
        }
        if !self.stack.is_empty() {
            writeln!(f, "  Stack: {} item(s)", self.stack.len())?;
        }
        Ok(())
    }
}
