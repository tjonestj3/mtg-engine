use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Color {
    White,
    Blue,
    Black,
    Red,
    Green,
}

impl Color {
    pub fn symbol(&self) -> char {
        match self {
            Color::White => 'W',
            Color::Blue => 'U',
            Color::Black => 'B',
            Color::Red => 'R',
            Color::Green => 'G',
        }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.symbol())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManaCost {
    pub generic: u8,
    pub white: u8,
    pub blue: u8,
    pub black: u8,
    pub red: u8,
    pub green: u8,
    pub colorless: u8,
}

impl ManaCost {
    pub fn new() -> Self {
        Self {
            generic: 0,
            white: 0,
            blue: 0,
            black: 0,
            red: 0,
            green: 0,
            colorless: 0,
        }
    }

    pub fn converted_mana_cost(&self) -> u8 {
        self.generic + self.white + self.blue + self.black + self.red + self.green + self.colorless
    }

    pub fn colors(&self) -> Vec<Color> {
        let mut colors = Vec::new();
        if self.white > 0 { colors.push(Color::White); }
        if self.blue > 0 { colors.push(Color::Blue); }
        if self.black > 0 { colors.push(Color::Black); }
        if self.red > 0 { colors.push(Color::Red); }
        if self.green > 0 { colors.push(Color::Green); }
        colors
    }
}

impl Default for ManaCost {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ManaCost {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.generic > 0 {
            write!(f, "{{{}}}", self.generic)?;
        }
        for _ in 0..self.white { write!(f, "{{W}}")?; }
        for _ in 0..self.blue { write!(f, "{{U}}")?; }
        for _ in 0..self.black { write!(f, "{{B}}")?; }
        for _ in 0..self.red { write!(f, "{{R}}")?; }
        for _ in 0..self.green { write!(f, "{{G}}")?; }
        for _ in 0..self.colorless { write!(f, "{{C}}")?; }
        Ok(())
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ManaPool {
    pub white: u8,
    pub blue: u8,
    pub black: u8,
    pub red: u8,
    pub green: u8,
    pub colorless: u8,
}

impl ManaPool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, color: Color, amount: u8) {
        match color {
            Color::White => self.white += amount,
            Color::Blue => self.blue += amount,
            Color::Black => self.black += amount,
            Color::Red => self.red += amount,
            Color::Green => self.green += amount,
        }
    }

    pub fn add_colorless(&mut self, amount: u8) {
        self.colorless += amount;
    }

    pub fn total(&self) -> u8 {
        self.white + self.blue + self.black + self.red + self.green + self.colorless
    }

    pub fn can_pay(&self, cost: &ManaCost) -> bool {
        if self.white < cost.white
            || self.blue < cost.blue
            || self.black < cost.black
            || self.red < cost.red
            || self.green < cost.green
        {
            return false;
        }

        let remaining = self.total()
            - cost.white
            - cost.blue
            - cost.black
            - cost.red
            - cost.green
            - cost.colorless.min(self.colorless);

        let colorless_debt = cost.colorless.saturating_sub(self.colorless);
        remaining >= cost.generic + colorless_debt
    }

    pub fn pay(&mut self, cost: &ManaCost) -> bool {
        if !self.can_pay(cost) {
            return false;
        }

        self.white -= cost.white;
        self.blue -= cost.blue;
        self.black -= cost.black;
        self.red -= cost.red;
        self.green -= cost.green;

        let colorless_paid = cost.colorless.min(self.colorless);
        self.colorless -= colorless_paid;

        let mut generic_remaining = cost.generic + cost.colorless - colorless_paid;
        let pool_order = [&mut self.colorless, &mut self.red, &mut self.green, &mut self.white, &mut self.black, &mut self.blue];
        for pool in pool_order {
            let take = generic_remaining.min(*pool);
            *pool -= take;
            generic_remaining -= take;
            if generic_remaining == 0 { break; }
        }

        true
    }

    pub fn empty(&mut self) {
        *self = Self::new();
    }
}

impl fmt::Display for ManaPool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();
        if self.white > 0 { parts.push(format!("{}W", self.white)); }
        if self.blue > 0 { parts.push(format!("{}U", self.blue)); }
        if self.black > 0 { parts.push(format!("{}B", self.black)); }
        if self.red > 0 { parts.push(format!("{}R", self.red)); }
        if self.green > 0 { parts.push(format!("{}G", self.green)); }
        if self.colorless > 0 { parts.push(format!("{}C", self.colorless)); }
        if parts.is_empty() {
            write!(f, "(empty)")
        } else {
            write!(f, "{}", parts.join(", "))
        }
    }
}
