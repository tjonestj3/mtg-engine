use macroquad::prelude::*;

use crate::card::{CardData, CardId, CardType, Keyword};
use crate::effect::PermanentState;
use crate::game::state::{GameState, GameStatus};
use crate::game::turns::Step;
use crate::player::PlayerId;
use crate::zone::Zone;

const WINDOW_W: f32 = 1280.0;
const WINDOW_H: f32 = 800.0;
const CARD_W: f32 = 85.0;
const CARD_H: f32 = 115.0;
const TAPPED_W: f32 = 115.0;
const TAPPED_H: f32 = 85.0;
const CARD_GAP: f32 = 6.0;
const FONT_SM: f32 = 14.0;
const FONT_MD: f32 = 18.0;
const FONT_LG: f32 = 24.0;

const BG_COLOR: Color = Color::new(0.08, 0.08, 0.12, 1.0);
const PANEL_COLOR: Color = Color::new(0.12, 0.12, 0.18, 1.0);
const PANEL_BORDER: Color = Color::new(0.25, 0.25, 0.35, 1.0);
const TEXT_COLOR: Color = WHITE;
const DIM_TEXT: Color = Color::new(0.6, 0.6, 0.6, 1.0);
const HIGHLIGHT_COLOR: Color = Color::new(1.0, 0.9, 0.2, 0.8);
const ACTIVE_PLAYER_COLOR: Color = Color::new(0.2, 0.8, 0.3, 1.0);
const BUTTON_COLOR: Color = Color::new(0.2, 0.2, 0.35, 1.0);
const BUTTON_HOVER: Color = Color::new(0.3, 0.3, 0.5, 1.0);
const BUTTON_DISABLED: Color = Color::new(0.15, 0.15, 0.2, 1.0);
const ATTACKER_GLOW: Color = Color::new(1.0, 0.3, 0.3, 0.7);
const BLOCKER_GLOW: Color = Color::new(0.3, 0.3, 1.0, 0.7);

// Layout regions (y coordinates)
const P2_INFO_Y: f32 = 0.0;
const P2_INFO_H: f32 = 35.0;
const P2_BATTLE_Y: f32 = 38.0;
const P2_BATTLE_H: f32 = 140.0;
const P2_HAND_Y: f32 = 180.0;
const P2_HAND_H: f32 = 125.0;
const PHASE_BAR_Y: f32 = 310.0;
const PHASE_BAR_H: f32 = 50.0;
const P1_BATTLE_Y: f32 = 365.0;
const P1_BATTLE_H: f32 = 140.0;
const P1_INFO_Y: f32 = 510.0;
const P1_INFO_H: f32 = 35.0;
const P1_HAND_Y: f32 = 555.0;
const P1_HAND_H: f32 = 125.0;
const MSG_Y: f32 = 690.0;
const BUTTON_ROW_Y: f32 = 730.0;

#[derive(Debug, Clone, PartialEq)]
enum UiPhase {
    Untapping,
    Drawing,
    Main,
    SelectAttackers,
    SelectBlockers { attacker_to_block: Option<CardId> },
    CombatDamage,
    EndStep,
    Cleanup,
    NextTurn,
    GameOver,
}

struct UiState {
    phase: UiPhase,
    selected_attackers: Vec<CardId>,
    blocker_assignments: Vec<(CardId, CardId)>,
    message: String,
    message_timer: f64,
    auto_timer: f64,
    hovered_card: Option<CardId>,
}

impl UiState {
    fn new() -> Self {
        Self {
            phase: UiPhase::Untapping,
            selected_attackers: Vec::new(),
            blocker_assignments: Vec::new(),
            message: String::new(),
            message_timer: 0.0,
            auto_timer: 0.0,
            hovered_card: None,
        }
    }

    fn set_message(&mut self, msg: &str) {
        self.message = msg.to_string();
        self.message_timer = 2.5;
    }
}

fn card_bg(card: &CardData) -> Color {
    if card.is_land() {
        if !card.basic_land_types.is_empty() {
            match card.basic_land_types[0].produces() {
                crate::mana::Color::White => Color::new(0.95, 0.92, 0.80, 1.0),
                crate::mana::Color::Blue => Color::new(0.25, 0.40, 0.70, 1.0),
                crate::mana::Color::Black => Color::new(0.25, 0.22, 0.28, 1.0),
                crate::mana::Color::Red => Color::new(0.70, 0.25, 0.20, 1.0),
                crate::mana::Color::Green => Color::new(0.20, 0.55, 0.25, 1.0),
            }
        } else {
            Color::new(0.50, 0.42, 0.28, 1.0)
        }
    } else {
        let colors = card.cost.colors();
        if colors.len() > 1 {
            Color::new(0.75, 0.65, 0.20, 1.0) // gold
        } else if let Some(c) = colors.first() {
            match c {
                crate::mana::Color::White => Color::new(0.92, 0.90, 0.80, 1.0),
                crate::mana::Color::Blue => Color::new(0.20, 0.35, 0.65, 1.0),
                crate::mana::Color::Black => Color::new(0.22, 0.20, 0.25, 1.0),
                crate::mana::Color::Red => Color::new(0.65, 0.22, 0.18, 1.0),
                crate::mana::Color::Green => Color::new(0.18, 0.50, 0.22, 1.0),
            }
        } else {
            Color::new(0.55, 0.55, 0.55, 1.0) // colorless
        }
    }
}

fn card_text_color(card: &CardData) -> Color {
    let bg = card_bg(card);
    let brightness = bg.r * 0.299 + bg.g * 0.587 + bg.b * 0.114;
    if brightness > 0.5 { BLACK } else { WHITE }
}

fn draw_card_at(
    card: &CardData,
    perm: Option<&PermanentState>,
    x: f32,
    y: f32,
    tapped: bool,
    highlighted: bool,
    glow: Option<Color>,
    sick: bool,
) -> (f32, f32, f32, f32) {
    let (w, h) = if tapped { (TAPPED_W, TAPPED_H) } else { (CARD_W, CARD_H) };

    if let Some(glow_color) = glow {
        draw_rectangle(x - 3.0, y - 3.0, w + 6.0, h + 6.0, glow_color);
    }

    if highlighted {
        draw_rectangle(x - 2.0, y - 2.0, w + 4.0, h + 4.0, HIGHLIGHT_COLOR);
    }

    let bg = card_bg(card);
    let tc = card_text_color(card);
    let alpha = if sick { 0.7 } else { 1.0 };
    let bg = Color::new(bg.r, bg.g, bg.b, alpha);

    draw_rectangle(x, y, w, h, bg);
    draw_rectangle_lines(x, y, w, h, 1.5, Color::new(0.0, 0.0, 0.0, 0.6));

    // Card name
    let name = if card.name.len() > 11 {
        &card.name[..11]
    } else {
        &card.name
    };
    draw_text(name, x + 3.0, y + 14.0, FONT_SM, tc);

    // Mana cost
    if card.cost.converted_mana_cost() > 0 {
        let cost_str = format!("{}", card.cost.converted_mana_cost());
        draw_text(&cost_str, x + w - 16.0, y + 14.0, FONT_SM, tc);
    }

    // Type line
    let type_str = if card.is_creature() {
        "Creature"
    } else if card.is_land() {
        "Land"
    } else if card.is_type(CardType::Instant) {
        "Instant"
    } else if card.is_type(CardType::Sorcery) {
        "Sorcery"
    } else if card.is_type(CardType::Enchantment) {
        "Enchant"
    } else if card.is_type(CardType::Artifact) {
        "Artifact"
    } else {
        ""
    };

    if !tapped {
        draw_text(type_str, x + 3.0, y + 55.0, FONT_SM - 2.0, Color::new(tc.r, tc.g, tc.b, 0.7));
    }

    // Keywords
    if !card.keywords.is_empty() {
        let kws: Vec<&str> = card.keywords.iter().take(3).map(|k| match k {
            Keyword::Flying => "Fly",
            Keyword::FirstStrike => "FS",
            Keyword::DoubleStrike => "DS",
            Keyword::Deathtouch => "DT",
            Keyword::Lifelink => "LL",
            Keyword::Trample => "Trmp",
            Keyword::Vigilance => "Vig",
            Keyword::Reach => "Rch",
            Keyword::Haste => "Hst",
            Keyword::Hexproof => "Hex",
            Keyword::Indestructible => "Ind",
            Keyword::Menace => "Men",
            Keyword::Flash => "Fls",
            Keyword::Defender => "Def",
        }).collect();
        let kw_y = if tapped { y + h - 20.0 } else { y + 70.0 };
        draw_text(&kws.join(" "), x + 3.0, kw_y, FONT_SM - 3.0, Color::new(tc.r, tc.g, tc.b, 0.6));
    }

    // P/T
    if let (Some(p), Some(t)) = (card.power, card.toughness) {
        let (pm, tm) = if let Some(perm) = perm {
            (
                perm.power_modifier + perm.counters.plus_one - perm.counters.minus_one,
                perm.toughness_modifier + perm.counters.plus_one - perm.counters.minus_one,
            )
        } else {
            (0, 0)
        };
        let dmg = perm.map(|p| p.damage_marked).unwrap_or(0);
        let pt_str = if dmg > 0 {
            format!("{}/{} ({})", p + pm, t + tm, dmg)
        } else {
            format!("{}/{}", p + pm, t + tm)
        };
        draw_text(&pt_str, x + 3.0, y + h - 5.0, FONT_MD, tc);
    }

    // Tapped indicator
    if tapped {
        draw_text("TAP", x + w - 30.0, y + h - 5.0, FONT_SM, Color::new(1.0, 0.5, 0.0, 0.8));
    }

    // Summoning sickness indicator
    if sick {
        draw_text("zzz", x + w - 28.0, y + 28.0, FONT_SM, Color::new(1.0, 1.0, 0.3, 0.6));
    }

    (x, y, w, h)
}

fn draw_card_back(x: f32, y: f32) {
    draw_rectangle(x, y, CARD_W, CARD_H, Color::new(0.35, 0.15, 0.15, 1.0));
    draw_rectangle_lines(x, y, CARD_W, CARD_H, 1.5, Color::new(0.5, 0.25, 0.25, 1.0));
    draw_rectangle(x + 8.0, y + 8.0, CARD_W - 16.0, CARD_H - 16.0, Color::new(0.45, 0.22, 0.22, 1.0));
    draw_text("MTG", x + 22.0, y + 62.0, FONT_LG, Color::new(0.7, 0.5, 0.3, 0.8));
}

struct Button {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    label: String,
    enabled: bool,
}

impl Button {
    fn new(label: &str, x: f32, y: f32, w: f32, enabled: bool) -> Self {
        Self {
            x,
            y,
            w,
            h: 36.0,
            label: label.to_string(),
            enabled,
        }
    }

    fn draw(&self) -> bool {
        let mouse = mouse_position();
        let hovered = self.enabled
            && mouse.0 >= self.x
            && mouse.0 <= self.x + self.w
            && mouse.1 >= self.y
            && mouse.1 <= self.y + self.h;

        let bg = if !self.enabled {
            BUTTON_DISABLED
        } else if hovered {
            BUTTON_HOVER
        } else {
            BUTTON_COLOR
        };

        draw_rectangle(self.x, self.y, self.w, self.h, bg);
        draw_rectangle_lines(self.x, self.y, self.w, self.h, 1.0, PANEL_BORDER);

        let tc = if self.enabled { TEXT_COLOR } else { DIM_TEXT };
        let text_w = self.label.len() as f32 * 8.0;
        draw_text(
            &self.label,
            self.x + (self.w - text_w) / 2.0,
            self.y + 24.0,
            FONT_MD,
            tc,
        );

        hovered && self.enabled && is_mouse_button_pressed(MouseButton::Left)
    }
}

fn point_in_rect(mx: f32, my: f32, x: f32, y: f32, w: f32, h: f32) -> bool {
    mx >= x && mx <= x + w && my >= y && my <= y + h
}

fn cards_start_x(count: usize, card_w: f32) -> f32 {
    let total = count as f32 * (card_w + CARD_GAP) - CARD_GAP;
    (WINDOW_W - total).max(0.0) / 2.0
}

fn do_untap(game: &mut GameState) {
    let active = game.turn.active_player;
    let bf = game.player(active).zones.battlefield.clone();
    for id in bf {
        if let Some(perm) = game.get_permanent_mut(id) {
            perm.untap();
            perm.summoning_sick = false;
        }
    }
    game.player_mut(active).reset_for_turn();
}

fn do_draw(game: &mut GameState) -> Option<String> {
    let active = game.turn.active_player;
    if game.turn.turn_number == 1 && active == 0 {
        return Some("First player skips draw.".into());
    }
    if let Some(card_id) = game.player_mut(active).zones.draw_from_library() {
        let name = game.get_card(card_id).map(|c| c.name.clone()).unwrap_or_default();
        Some(format!("Drew: {}", name))
    } else {
        Some("Library empty — cannot draw!".into())
    }
}

fn do_cleanup(game: &mut GameState) {
    for player in &mut game.players {
        player.mana_pool.empty();
    }
    let all_perms: Vec<CardId> = game.permanents.keys().copied().collect();
    for id in all_perms {
        if let Some(perm) = game.get_permanent_mut(id) {
            perm.damage_marked = 0;
        }
    }
}

fn resolve_combat(game: &mut GameState, attackers: &[CardId], blockers: &[(CardId, CardId)], defending: PlayerId) -> Vec<String> {
    let mut log = Vec::new();

    for &attacker_id in attackers {
        let a_card = match game.get_card(attacker_id) { Some(c) => c.clone(), None => continue };
        let a_perm = match game.get_permanent(attacker_id) { Some(p) => p.clone(), None => continue };

        let a_power = a_card.power.unwrap_or(0) + a_perm.power_modifier + a_perm.counters.plus_one - a_perm.counters.minus_one;
        if a_power <= 0 { continue; }

        let assigned_blockers: Vec<CardId> = blockers.iter().filter(|(_, a)| *a == attacker_id).map(|(b, _)| *b).collect();

        if assigned_blockers.is_empty() {
            game.player_mut(defending).deal_damage(a_power);
            log.push(format!("{} hits for {} (life: {})", a_card.name, a_power, game.player(defending).life));

            if a_card.has_keyword(Keyword::Lifelink) {
                let ctrl = find_controller(game, attacker_id);
                game.player_mut(ctrl).gain_life(a_power);
                log.push(format!("  Lifelink +{}", a_power));
            }
        } else {
            let mut remaining = a_power;
            let has_trample = a_card.has_keyword(Keyword::Trample);
            let has_deathtouch = a_card.has_keyword(Keyword::Deathtouch);

            for &blocker_id in &assigned_blockers {
                let b_card = match game.get_card(blocker_id) { Some(c) => c.clone(), None => continue };
                let b_perm = match game.get_permanent(blocker_id) { Some(p) => p.clone(), None => continue };

                let b_power = b_card.power.unwrap_or(0) + b_perm.power_modifier;
                let b_toughness = b_card.toughness.unwrap_or(0) + b_perm.toughness_modifier - b_perm.damage_marked;

                let dmg = if has_deathtouch { 1.min(remaining) } else { b_toughness.max(0).min(remaining) };

                if let Some(perm) = game.get_permanent_mut(blocker_id) { perm.damage_marked += dmg; }
                remaining -= dmg;
                log.push(format!("{} deals {} to {}", a_card.name, dmg, b_card.name));

                if b_power > 0 {
                    if let Some(perm) = game.get_permanent_mut(attacker_id) { perm.damage_marked += b_power; }
                    log.push(format!("{} deals {} to {}", b_card.name, b_power, a_card.name));
                }
            }

            if has_trample && remaining > 0 {
                game.player_mut(defending).deal_damage(remaining);
                log.push(format!("{} tramples for {} (life: {})", a_card.name, remaining, game.player(defending).life));
            }
        }
    }

    game.check_state_based_actions();
    log
}

fn find_controller(game: &GameState, card_id: CardId) -> PlayerId {
    for p in &game.players {
        if p.zones.battlefield.contains(&card_id) { return p.id; }
    }
    0
}

fn can_play_any_land(game: &GameState, pid: PlayerId) -> bool {
    game.player(pid).can_play_land()
        && game.player(pid).zones.hand.iter().any(|&id| game.get_card(id).map(|c| c.is_land()).unwrap_or(false))
}

fn can_cast_any(game: &GameState, pid: PlayerId) -> bool {
    game.player(pid).zones.hand.iter().any(|&id| {
        game.get_card(id).map(|c| !c.is_land() && game.player(pid).mana_pool.can_pay(&c.cost)).unwrap_or(false)
    })
}

fn has_untapped_lands(game: &GameState, pid: PlayerId) -> bool {
    game.player(pid).zones.battlefield.iter().any(|&id| {
        let is_land = game.get_card(id).map(|c| c.is_land() && !c.basic_land_types.is_empty()).unwrap_or(false);
        let untapped = game.get_permanent(id).map(|p| !p.tapped).unwrap_or(false);
        is_land && untapped
    })
}

fn has_eligible_attackers(game: &GameState, pid: PlayerId) -> bool {
    game.player(pid).zones.battlefield.iter().any(|&id| {
        let card = game.get_card(id);
        let perm = game.get_permanent(id);
        match (card, perm) {
            (Some(c), Some(p)) => c.is_creature() && p.can_attack() && !c.has_keyword(Keyword::Defender),
            _ => false,
        }
    })
}

pub async fn run_game_ui(game: &mut GameState) {
    // Draw opening hands
    for pid in 0..game.players.len() as u8 {
        for _ in 0..7 {
            game.player_mut(pid).zones.draw_from_library();
        }
    }

    let mut ui = UiState::new();
    ui.auto_timer = 0.8;
    ui.set_message("Game started! Opening hands drawn.");
    let mut combat_log: Vec<String> = Vec::new();

    loop {
        let dt = get_frame_time() as f64;
        let active = game.turn.active_player;
        let defending = (active + 1) % game.players.len() as u8;
        let mouse = mouse_position();

        // Tick timers
        if ui.message_timer > 0.0 { ui.message_timer -= dt; }
        if ui.auto_timer > 0.0 { ui.auto_timer -= dt; }

        // === Phase logic ===
        match ui.phase.clone() {
            UiPhase::Untapping => {
                if ui.auto_timer <= 0.0 {
                    game.turn.step = Step::Untap;
                    do_untap(game);
                    ui.set_message(&format!("{}: Untap", game.player(active).name));
                    ui.phase = UiPhase::Drawing;
                    ui.auto_timer = 0.6;
                }
            }
            UiPhase::Drawing => {
                if ui.auto_timer <= 0.0 {
                    game.turn.step = Step::Draw;
                    if let Some(msg) = do_draw(game) {
                        ui.set_message(&msg);
                    }
                    ui.phase = UiPhase::Main;
                    game.turn.step = Step::Main;
                    game.turn.is_precombat_main = true;
                }
            }
            UiPhase::Main => {
                // Input handled in draw section via buttons/clicks
            }
            UiPhase::SelectAttackers => {}
            UiPhase::SelectBlockers { .. } => {}
            UiPhase::CombatDamage => {
                if ui.auto_timer <= 0.0 {
                    combat_log.clear();
                    if game.turn.is_precombat_main {
                        game.turn.is_precombat_main = false;
                    }
                    ui.phase = UiPhase::Main;
                    game.turn.step = Step::Main;
                }
            }
            UiPhase::EndStep => {
                // End step with priority — pass moves to cleanup
            }
            UiPhase::Cleanup => {
                if ui.auto_timer <= 0.0 {
                    do_cleanup(game);
                    // Discard to 7 (auto-discard last cards for simplicity in v1)
                    while game.player(active).hand_size() > 7 {
                        let last = *game.player(active).zones.hand.last().unwrap();
                        game.player_mut(active).zones.move_card(last, Zone::Hand, Zone::Graveyard);
                    }
                    ui.phase = UiPhase::NextTurn;
                    ui.auto_timer = 0.3;
                }
            }
            UiPhase::NextTurn => {
                if ui.auto_timer <= 0.0 {
                    game.turn.end_turn(game.players.len() as u8);
                    ui.phase = UiPhase::Untapping;
                    ui.auto_timer = 0.5;
                }
            }
            UiPhase::GameOver => {}
        }

        // Check game over
        if game.status != GameStatus::InProgress && ui.phase != UiPhase::GameOver {
            ui.phase = UiPhase::GameOver;
            match game.status {
                GameStatus::Won(id) => ui.set_message(&format!("{} wins!", game.player(id).name)),
                GameStatus::Draw => ui.set_message("Draw!"),
                _ => {}
            }
        }

        // === DRAWING ===
        clear_background(BG_COLOR);

        // -- P2 info bar --
        draw_rectangle(0.0, P2_INFO_Y, WINDOW_W, P2_INFO_H, PANEL_COLOR);
        let p2_color = if active == 1 { ACTIVE_PLAYER_COLOR } else { DIM_TEXT };
        let p2 = game.player(1);
        draw_text(
            &format!("{} — Life: {} | Hand: {} | Library: {} | Mana: {}", p2.name, p2.life, p2.hand_size(), p2.library_size(), p2.mana_pool),
            10.0, P2_INFO_Y + 24.0, FONT_MD, p2_color,
        );

        // -- P2 hand (card backs if P1 active, face up if P2 active) --
        {
            let hand = game.player(1).zones.hand.clone();
            let start_x = cards_start_x(hand.len(), CARD_W);
            for (i, &cid) in hand.iter().enumerate() {
                let cx = start_x + i as f32 * (CARD_W + CARD_GAP);
                let cy = P2_HAND_Y;

                if active == 1 {
                    let card = game.get_card(cid).cloned();
                    let hovered = point_in_rect(mouse.0, mouse.1, cx, cy, CARD_W, CARD_H);

                    if let Some(card) = card {
                        let glow = if hovered && ui.phase == UiPhase::Main { Some(HIGHLIGHT_COLOR) } else { None };
                        draw_card_at(&card, None, cx, cy, false, false, glow, false);

                        if hovered && is_mouse_button_pressed(MouseButton::Left) && ui.phase == UiPhase::Main {
                            if card.is_land() && game.player(1).can_play_land() {
                                game.player_mut(1).zones.move_card(cid, Zone::Hand, Zone::Battlefield);
                                let mut perm = PermanentState::new(cid);
                                perm.summoning_sick = false;
                                game.permanents.insert(cid, perm);
                                game.player_mut(1).land_plays_remaining -= 1;
                                ui.set_message(&format!("Played {}", card.name));
                            } else if !card.is_land() && game.player(1).mana_pool.can_pay(&card.cost) {
                                let cost = card.cost.clone();
                                let name = card.name.clone();
                                let is_creature = card.is_creature();
                                let has_haste = card.has_keyword(Keyword::Haste);
                                game.player_mut(1).mana_pool.pay(&cost);
                                game.player_mut(1).zones.move_card(cid, Zone::Hand, Zone::Battlefield);
                                let mut perm = PermanentState::new(cid);
                                if has_haste || !is_creature { perm.summoning_sick = false; }
                                game.permanents.insert(cid, perm);
                                game.check_state_based_actions();
                                ui.set_message(&format!("Cast {}!", name));
                            }
                        }
                    }
                } else {
                    draw_card_back(cx, cy);
                }
            }
        }

        // -- P2 battlefield --
        draw_rectangle(0.0, P2_BATTLE_Y, WINDOW_W, P2_BATTLE_H, Color::new(0.10, 0.10, 0.15, 1.0));
        draw_line(0.0, P2_BATTLE_Y, WINDOW_W, P2_BATTLE_Y, 1.0, PANEL_BORDER);
        draw_line(0.0, P2_BATTLE_Y + P2_BATTLE_H, WINDOW_W, P2_BATTLE_Y + P2_BATTLE_H, 1.0, PANEL_BORDER);
        {
            let bf = game.player(1).zones.battlefield.clone();
            let start_x = cards_start_x(bf.len(), CARD_W);
            for (i, &cid) in bf.iter().enumerate() {
                let card = game.get_card(cid).cloned();
                let perm = game.get_permanent(cid).cloned();
                if let (Some(card), Some(perm)) = (card, perm) {
                    let tapped = perm.tapped;
                    let cw = if tapped { TAPPED_W } else { CARD_W };
                    let ch = if tapped { TAPPED_H } else { CARD_H };
                    let cx = start_x + i as f32 * (TAPPED_W + CARD_GAP);
                    let cy = P2_BATTLE_Y + (P2_BATTLE_H - ch) / 2.0;
                    let sick = perm.summoning_sick && card.is_creature();

                    let glow = if ui.selected_attackers.contains(&cid) {
                        Some(ATTACKER_GLOW)
                    } else if ui.blocker_assignments.iter().any(|(b, _)| *b == cid) {
                        Some(BLOCKER_GLOW)
                    } else {
                        None
                    };

                    let hovered = point_in_rect(mouse.0, mouse.1, cx, cy, cw, ch);

                    draw_card_at(&card, Some(&perm), cx, cy, tapped, false, glow, sick);

                    if hovered && is_mouse_button_pressed(MouseButton::Left) {
                        if active == 1 && ui.phase == UiPhase::Main && card.is_land() && !perm.tapped && !card.basic_land_types.is_empty() {
                            let types = card.basic_land_types.clone();
                            if let Some(p) = game.get_permanent_mut(cid) { p.tap(); }
                            for lt in &types { game.player_mut(1).mana_pool.add(lt.produces(), 1); }
                            ui.set_message(&format!("Tapped {} for mana", card.name));
                        }
                        if let UiPhase::SelectBlockers { attacker_to_block } = &ui.phase {
                            if active == 0 && card.is_creature() && !perm.tapped {
                                if let Some(atk_id) = attacker_to_block {
                                    ui.blocker_assignments.push((cid, *atk_id));
                                    ui.set_message(&format!("{} blocks", card.name));
                                    ui.phase = UiPhase::SelectBlockers { attacker_to_block: None };
                                }
                            }
                        }
                    }
                }
            }
        }

        // -- Phase bar --
        draw_rectangle(0.0, PHASE_BAR_Y, WINDOW_W, PHASE_BAR_H, Color::new(0.15, 0.15, 0.22, 1.0));
        draw_line(0.0, PHASE_BAR_Y, WINDOW_W, PHASE_BAR_Y, 1.0, PANEL_BORDER);
        draw_line(0.0, PHASE_BAR_Y + PHASE_BAR_H, WINDOW_W, PHASE_BAR_Y + PHASE_BAR_H, 1.0, PANEL_BORDER);

        let phase_name = match &ui.phase {
            UiPhase::Untapping => "Untap",
            UiPhase::Drawing => "Draw",
            UiPhase::Main if game.turn.is_precombat_main => "Precombat Main",
            UiPhase::Main => "Postcombat Main",
            UiPhase::SelectAttackers => "Declare Attackers",
            UiPhase::SelectBlockers { .. } => "Declare Blockers",
            UiPhase::CombatDamage => "Combat Damage",
            UiPhase::EndStep => "End Step",
            UiPhase::Cleanup => "Cleanup",
            UiPhase::NextTurn => "Next Turn",
            UiPhase::GameOver => "Game Over",
        };

        draw_text(
            &format!("Turn {} — {} — {}", game.turn.turn_number, game.player(active).name, phase_name),
            10.0, PHASE_BAR_Y + 32.0, FONT_LG, ACTIVE_PLAYER_COLOR,
        );

        // -- P1 battlefield --
        draw_rectangle(0.0, P1_BATTLE_Y, WINDOW_W, P1_BATTLE_H, Color::new(0.10, 0.10, 0.15, 1.0));
        draw_line(0.0, P1_BATTLE_Y, WINDOW_W, P1_BATTLE_Y, 1.0, PANEL_BORDER);
        draw_line(0.0, P1_BATTLE_Y + P1_BATTLE_H, WINDOW_W, P1_BATTLE_Y + P1_BATTLE_H, 1.0, PANEL_BORDER);
        {
            let bf = game.player(0).zones.battlefield.clone();
            let start_x = cards_start_x(bf.len(), CARD_W);
            for (i, &cid) in bf.iter().enumerate() {
                let card = game.get_card(cid).cloned();
                let perm = game.get_permanent(cid).cloned();
                if let (Some(card), Some(perm)) = (card, perm) {
                    let tapped = perm.tapped;
                    let cw = if tapped { TAPPED_W } else { CARD_W };
                    let ch = if tapped { TAPPED_H } else { CARD_H };
                    let cx = start_x + i as f32 * (TAPPED_W + CARD_GAP);
                    let cy = P1_BATTLE_Y + (P1_BATTLE_H - ch) / 2.0;
                    let sick = perm.summoning_sick && card.is_creature();

                    let glow = if ui.selected_attackers.contains(&cid) {
                        Some(ATTACKER_GLOW)
                    } else if ui.blocker_assignments.iter().any(|(b, _)| *b == cid) {
                        Some(BLOCKER_GLOW)
                    } else {
                        None
                    };

                    let hovered = point_in_rect(mouse.0, mouse.1, cx, cy, cw, ch);

                    draw_card_at(&card, Some(&perm), cx, cy, tapped, false, glow, sick);

                    if hovered && is_mouse_button_pressed(MouseButton::Left) {
                        if active == 0 && ui.phase == UiPhase::Main && card.is_land() && !perm.tapped && !card.basic_land_types.is_empty() {
                            let types = card.basic_land_types.clone();
                            if let Some(p) = game.get_permanent_mut(cid) { p.tap(); }
                            for lt in &types { game.player_mut(0).mana_pool.add(lt.produces(), 1); }
                            ui.set_message(&format!("Tapped {} for mana", card.name));
                        }
                        if ui.phase == UiPhase::SelectAttackers && active == 0 && card.is_creature() && perm.can_attack() && !card.has_keyword(Keyword::Defender) {
                            if ui.selected_attackers.contains(&cid) {
                                ui.selected_attackers.retain(|&id| id != cid);
                            } else {
                                ui.selected_attackers.push(cid);
                            }
                        }
                        if let UiPhase::SelectBlockers { attacker_to_block } = &ui.phase {
                            if active == 1 && card.is_creature() && !perm.tapped {
                                if let Some(atk_id) = attacker_to_block {
                                    ui.blocker_assignments.push((cid, *atk_id));
                                    ui.set_message(&format!("{} blocks", card.name));
                                    ui.phase = UiPhase::SelectBlockers { attacker_to_block: None };
                                }
                            }
                        }
                    }
                }
            }
        }

        // -- P1 info bar --
        draw_rectangle(0.0, P1_INFO_Y, WINDOW_W, P1_INFO_H, PANEL_COLOR);
        let p1_color = if active == 0 { ACTIVE_PLAYER_COLOR } else { DIM_TEXT };
        let p1 = game.player(0);
        draw_text(
            &format!("{} — Life: {} | Hand: {} | Library: {} | Mana: {}", p1.name, p1.life, p1.hand_size(), p1.library_size(), p1.mana_pool),
            10.0, P1_INFO_Y + 24.0, FONT_MD, p1_color,
        );

        // -- P1 hand --
        {
            let hand = game.player(0).zones.hand.clone();
            let start_x = cards_start_x(hand.len(), CARD_W);
            for (i, &cid) in hand.iter().enumerate() {
                let cx = start_x + i as f32 * (CARD_W + CARD_GAP);
                let cy = P1_HAND_Y;

                if active == 0 {
                    let card = game.get_card(cid).cloned();
                    let hovered = point_in_rect(mouse.0, mouse.1, cx, cy, CARD_W, CARD_H);

                    if let Some(card) = card {
                        let glow = if hovered && ui.phase == UiPhase::Main { Some(HIGHLIGHT_COLOR) } else { None };
                        draw_card_at(&card, None, cx, cy, false, false, glow, false);

                        if hovered && is_mouse_button_pressed(MouseButton::Left) && ui.phase == UiPhase::Main {
                            if card.is_land() && game.player(0).can_play_land() {
                                game.player_mut(0).zones.move_card(cid, Zone::Hand, Zone::Battlefield);
                                let mut perm = PermanentState::new(cid);
                                perm.summoning_sick = false;
                                game.permanents.insert(cid, perm);
                                game.player_mut(0).land_plays_remaining -= 1;
                                ui.set_message(&format!("Played {}", card.name));
                            } else if !card.is_land() && game.player(0).mana_pool.can_pay(&card.cost) {
                                let cost = card.cost.clone();
                                let name = card.name.clone();
                                let is_creature = card.is_creature();
                                let has_haste = card.has_keyword(Keyword::Haste);
                                game.player_mut(0).mana_pool.pay(&cost);
                                game.player_mut(0).zones.move_card(cid, Zone::Hand, Zone::Battlefield);
                                let mut perm = PermanentState::new(cid);
                                if has_haste || !is_creature { perm.summoning_sick = false; }
                                game.permanents.insert(cid, perm);
                                game.check_state_based_actions();
                                ui.set_message(&format!("Cast {}!", name));
                            }
                        }
                    }
                } else {
                    draw_card_back(cx, cy);
                }
            }
        }

        // -- Message bar --
        if ui.message_timer > 0.0 {
            let alpha = (ui.message_timer.min(1.0)) as f32;
            draw_text(&ui.message, 10.0, MSG_Y + 20.0, FONT_MD, Color::new(1.0, 0.95, 0.7, alpha));
        }

        // -- Combat log --
        if ui.phase == UiPhase::CombatDamage && !combat_log.is_empty() {
            let log_x = WINDOW_W - 350.0;
            draw_rectangle(log_x - 10.0, PHASE_BAR_Y + PHASE_BAR_H + 5.0, 350.0, combat_log.len() as f32 * 18.0 + 10.0, Color::new(0.0, 0.0, 0.0, 0.7));
            for (i, line) in combat_log.iter().enumerate() {
                draw_text(line, log_x, PHASE_BAR_Y + PHASE_BAR_H + 22.0 + i as f32 * 18.0, FONT_SM, Color::new(1.0, 0.8, 0.8, 1.0));
            }
        }

        // -- Action buttons --
        match &ui.phase {
            UiPhase::Main => {
                let btn_y = BUTTON_ROW_Y;
                let combat_btn = Button::new("Combat", 20.0, btn_y, 110.0, has_eligible_attackers(game, active));
                let pass_btn = Button::new("Pass Turn", 140.0, btn_y, 110.0, true);
                let endstep_btn = Button::new("End Step", 260.0, btn_y, 110.0, true);

                if combat_btn.draw() {
                    ui.phase = UiPhase::SelectAttackers;
                    ui.selected_attackers.clear();
                    ui.set_message("Click creatures to attack, then Done.");
                }
                if pass_btn.draw() {
                    if game.turn.is_precombat_main {
                        game.turn.is_precombat_main = false;
                        ui.set_message("Postcombat Main Phase");
                    } else {
                        ui.phase = UiPhase::EndStep;
                        game.turn.step = Step::EndStep;
                        ui.set_message("End Step — pass priority?");
                    }
                }
                if endstep_btn.draw() {
                    ui.phase = UiPhase::EndStep;
                    game.turn.step = Step::EndStep;
                    ui.set_message("End Step — pass priority?");
                }
            }
            UiPhase::SelectAttackers => {
                let done_btn = Button::new("Done", 20.0, BUTTON_ROW_Y, 110.0, true);
                let cancel_btn = Button::new("Cancel", 140.0, BUTTON_ROW_Y, 110.0, true);

                if done_btn.draw() {
                    // Tap attackers
                    for &aid in &ui.selected_attackers {
                        let has_vig = game.get_card(aid).map(|c| c.has_keyword(Keyword::Vigilance)).unwrap_or(false);
                        if !has_vig {
                            if let Some(p) = game.get_permanent_mut(aid) { p.tap(); }
                        }
                    }

                    if ui.selected_attackers.is_empty() {
                        ui.phase = UiPhase::Main;
                        game.turn.is_precombat_main = false;
                        ui.set_message("No attackers — Postcombat Main");
                    } else {
                        ui.phase = UiPhase::SelectBlockers { attacker_to_block: None };
                        ui.blocker_assignments.clear();
                        ui.set_message(&format!("{}: Click an attacker, then a blocker.", game.player(defending).name));
                    }
                }
                if cancel_btn.draw() {
                    ui.selected_attackers.clear();
                    ui.phase = UiPhase::Main;
                }
            }
            UiPhase::SelectBlockers { .. } => {
                // Show attacker buttons to select which one to block
                let attackers = ui.selected_attackers.clone();
                let mut btn_x = 20.0;
                for &aid in &attackers {
                    let name = game.get_card(aid).map(|c| c.name.clone()).unwrap_or_default();
                    let short = if name.len() > 10 { &name[..10] } else { &name };
                    let btn = Button::new(&format!("Block {}", short), btn_x, BUTTON_ROW_Y, 130.0, true);
                    if btn.draw() {
                        ui.phase = UiPhase::SelectBlockers { attacker_to_block: Some(aid) };
                        ui.set_message(&format!("Click a creature to block {}", name));
                    }
                    btn_x += 138.0;
                }

                let done_btn = Button::new("Done", btn_x, BUTTON_ROW_Y, 110.0, true);
                if done_btn.draw() {
                    let attackers = ui.selected_attackers.clone();
                    let blockers = ui.blocker_assignments.clone();
                    combat_log = resolve_combat(game, &attackers, &blockers, defending);
                    ui.selected_attackers.clear();
                    ui.blocker_assignments.clear();
                    ui.phase = UiPhase::CombatDamage;
                    ui.auto_timer = 2.5;
                    ui.set_message("Combat resolved!");
                }
            }
            UiPhase::EndStep => {
                let pass_btn = Button::new("Pass", 20.0, BUTTON_ROW_Y, 110.0, true);
                if pass_btn.draw() {
                    ui.phase = UiPhase::Cleanup;
                    game.turn.step = Step::Cleanup;
                    ui.auto_timer = 0.3;
                }
            }
            UiPhase::GameOver => {
                let quit_btn = Button::new("Quit", 20.0, BUTTON_ROW_Y, 110.0, true);
                let again_btn = Button::new("New Game", 140.0, BUTTON_ROW_Y, 110.0, true);
                if quit_btn.draw() {
                    break;
                }
                if again_btn.draw() {
                    // Would need full reset — for now just quit
                    break;
                }
            }
            _ => {}
        }

        // Graveyard counts
        for pid in 0..2u8 {
            let gy = &game.player(pid).zones.graveyard;
            if !gy.is_empty() {
                let gy_y = if pid == 1 { P2_BATTLE_Y + 2.0 } else { P1_BATTLE_Y + P1_BATTLE_H - 16.0 };
                draw_text(&format!("GY: {}", gy.len()), WINDOW_W - 60.0, gy_y, FONT_SM, DIM_TEXT);
            }
        }

        next_frame().await;
    }
}
