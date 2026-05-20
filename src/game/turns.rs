use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    Beginning,
    PrecombatMain,
    Combat,
    PostcombatMain,
    Ending,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Step {
    Untap,
    Upkeep,
    Draw,
    Main,
    BeginCombat,
    DeclareAttackers,
    DeclareBlockers,
    CombatDamage,
    EndOfCombat,
    EndStep,
    Cleanup,
}

impl Step {
    pub fn phase(&self) -> Phase {
        match self {
            Step::Untap | Step::Upkeep | Step::Draw => Phase::Beginning,
            Step::Main => Phase::PrecombatMain, // or PostcombatMain depending on context
            Step::BeginCombat | Step::DeclareAttackers | Step::DeclareBlockers
            | Step::CombatDamage | Step::EndOfCombat => Phase::Combat,
            Step::EndStep | Step::Cleanup => Phase::Ending,
        }
    }

    pub fn allows_sorcery_speed(&self) -> bool {
        matches!(self, Step::Main)
    }

    pub fn next(&self, is_precombat_main: bool) -> Option<Step> {
        match self {
            Step::Untap => Some(Step::Upkeep),
            Step::Upkeep => Some(Step::Draw),
            Step::Draw => Some(Step::Main), // precombat main
            Step::Main if is_precombat_main => Some(Step::BeginCombat),
            Step::Main => Some(Step::EndStep), // postcombat main -> end
            Step::BeginCombat => Some(Step::DeclareAttackers),
            Step::DeclareAttackers => Some(Step::DeclareBlockers),
            Step::DeclareBlockers => Some(Step::CombatDamage),
            Step::CombatDamage => Some(Step::EndOfCombat),
            Step::EndOfCombat => Some(Step::Main), // postcombat main
            Step::EndStep => Some(Step::Cleanup),
            Step::Cleanup => None, // turn ends
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnState {
    pub turn_number: u32,
    pub active_player: u8,
    pub step: Step,
    pub is_precombat_main: bool,
    pub priority_player: u8,
    pub priority_passed_count: u8,
}

impl TurnState {
    pub fn new() -> Self {
        Self {
            turn_number: 1,
            active_player: 0,
            step: Step::Untap,
            is_precombat_main: true,
            priority_player: 0,
            priority_passed_count: 0,
        }
    }

    pub fn advance_step(&mut self) -> bool {
        if let Some(next) = self.step.next(self.is_precombat_main) {
            if self.step == Step::Main && self.is_precombat_main {
                self.is_precombat_main = false;
            }
            if next == Step::Main && !self.is_precombat_main {
                // Entering postcombat main
            }
            self.step = next;
            self.priority_passed_count = 0;
            true
        } else {
            false
        }
    }

    pub fn end_turn(&mut self, num_players: u8) {
        self.turn_number += 1;
        self.active_player = (self.active_player + 1) % num_players;
        self.step = Step::Untap;
        self.is_precombat_main = true;
        self.priority_player = self.active_player;
        self.priority_passed_count = 0;
    }
}

impl Default for TurnState {
    fn default() -> Self {
        Self::new()
    }
}
