use serde::{Deserialize, Serialize};

use crate::ability::Effect;
use crate::card::CardId;
use crate::player::PlayerId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StackItemKind {
    Spell { card_id: CardId },
    ActivatedAbility { source: CardId, effects: Vec<Effect> },
    TriggeredAbility { source: CardId, effects: Vec<Effect> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackItem {
    pub kind: StackItemKind,
    pub controller: PlayerId,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GameStack {
    items: Vec<StackItem>,
}

impl GameStack {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, item: StackItem) {
        self.items.push(item);
    }

    pub fn pop(&mut self) -> Option<StackItem> {
        self.items.pop()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn peek(&self) -> Option<&StackItem> {
        self.items.last()
    }

    pub fn items(&self) -> &[StackItem] {
        &self.items
    }
}
