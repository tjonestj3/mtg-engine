use std::io::{self, Write};

use crate::card::{CardId, Keyword};
use crate::effect::PermanentState;
use crate::game::state::{GameState, GameStatus};
use crate::game::turns::Step;
use crate::player::PlayerId;
use crate::zone::Zone;

fn prompt(msg: &str) -> String {
    print!("{}", msg);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_lowercase()
}

fn prompt_number(msg: &str, max: usize) -> Option<usize> {
    let input = prompt(msg);
    if input == "b" || input == "back" || input.is_empty() {
        return None;
    }
    input.parse::<usize>().ok().filter(|&n| n >= 1 && n <= max)
}

fn display_battlefield(game: &GameState, player_id: PlayerId) {
    let player = game.player(player_id);
    let perms: Vec<String> = player
        .zones
        .battlefield
        .iter()
        .filter_map(|&id| {
            let card = game.get_card(id)?;
            let perm = game.get_permanent(id)?;
            let mut parts = vec![card.name.clone()];
            if let (Some(p), Some(t)) = (card.power, card.toughness) {
                let eff_p = p + perm.power_modifier + perm.counters.plus_one - perm.counters.minus_one;
                let eff_t = t + perm.toughness_modifier + perm.counters.plus_one - perm.counters.minus_one;
                if perm.damage_marked > 0 {
                    parts.push(format!("{}/{} ({}dmg)", eff_p, eff_t, perm.damage_marked));
                } else {
                    parts.push(format!("{}/{}", eff_p, eff_t));
                }
            }
            if perm.tapped {
                parts.push("(T)".into());
            }
            if perm.summoning_sick && card.is_creature() {
                parts.push("(sick)".into());
            }
            if !card.keywords.is_empty() {
                let kws: Vec<&str> = card.keywords.iter().map(|k| keyword_short(k)).collect();
                parts.push(format!("[{}]", kws.join(",")));
            }
            Some(parts.join(" "))
        })
        .collect();

    if perms.is_empty() {
        println!("    Battlefield: (empty)");
    } else {
        println!("    Battlefield: {}", perms.join(" | "));
    }
}

fn display_graveyard(game: &GameState, player_id: PlayerId) {
    let gy = &game.player(player_id).zones.graveyard;
    if !gy.is_empty() {
        let names: Vec<String> = gy
            .iter()
            .filter_map(|&id| game.get_card(id).map(|c| c.name.clone()))
            .collect();
        println!("    Graveyard: {}", names.join(", "));
    }
}

fn display_game_state(game: &GameState) {
    let phase_name = match game.turn.step {
        Step::Main if game.turn.is_precombat_main => "Precombat Main",
        Step::Main => "Postcombat Main",
        other => match other {
            Step::Untap => "Untap",
            Step::Upkeep => "Upkeep",
            Step::Draw => "Draw",
            Step::BeginCombat => "Begin Combat",
            Step::DeclareAttackers => "Declare Attackers",
            Step::DeclareBlockers => "Declare Blockers",
            Step::CombatDamage => "Combat Damage",
            Step::EndOfCombat => "End of Combat",
            Step::EndStep => "End Step",
            Step::Cleanup => "Cleanup",
            _ => "Main",
        },
    };

    println!();
    println!("══════════════════════════════════════════════════");
    println!(
        "  Turn {} — {} ({})",
        game.turn.turn_number,
        phase_name,
        game.active_player().name,
    );
    println!("══════════════════════════════════════════════════");

    for p in &game.players {
        let active = if p.id == game.turn.active_player { " ◄" } else { "" };
        println!(
            "  {} — Life: {} | Hand: {} | Library: {} | Mana: {}{}",
            p.name,
            p.life,
            p.hand_size(),
            p.library_size(),
            p.mana_pool,
            active,
        );
        display_battlefield(game, p.id);
        display_graveyard(game, p.id);
    }
    println!("──────────────────────────────────────────────────");
}

fn display_hand(game: &GameState, player_id: PlayerId) {
    let player = game.player(player_id);
    println!();
    println!("{}'s hand:", player.name);
    for (i, &card_id) in player.zones.hand.iter().enumerate() {
        if let Some(card) = game.get_card(card_id) {
            let castable = if card.is_land() {
                if player.can_play_land() { " [can play]" } else { "" }
            } else if player.mana_pool.can_pay(&card.cost) {
                " [can cast]"
            } else {
                ""
            };
            println!("  [{}] {}{}", i + 1, card, castable);
        }
    }
}

fn keyword_short(kw: &Keyword) -> &'static str {
    match kw {
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
    }
}

fn tap_land_for_mana(game: &mut GameState, player_id: PlayerId) {
    let untapped_lands: Vec<(usize, CardId, String)> = game
        .player(player_id)
        .zones
        .battlefield
        .iter()
        .enumerate()
        .filter_map(|(i, &id)| {
            let card = game.get_card(id)?;
            let perm = game.get_permanent(id)?;
            if card.is_land() && !perm.tapped && !card.basic_land_types.is_empty() {
                Some((i, id, card.to_string()))
            } else {
                None
            }
        })
        .collect();

    if untapped_lands.is_empty() {
        println!("  No untapped lands to tap.");
        return;
    }

    println!("  Tap which land?");
    for (idx, (_, _, name)) in untapped_lands.iter().enumerate() {
        println!("    [{}] {}", idx + 1, name);
    }
    println!("    [B] Back");

    if let Some(choice) = prompt_number("  > ", untapped_lands.len()) {
        let (_, card_id, _) = &untapped_lands[choice - 1];
        let card_id = *card_id;

        let land_types = game
            .get_card(card_id)
            .map(|c| c.basic_land_types.clone())
            .unwrap_or_default();

        if let Some(perm) = game.get_permanent_mut(card_id) {
            perm.tap();
        }

        for lt in &land_types {
            game.player_mut(player_id).mana_pool.add(lt.produces(), 1);
        }

        let name = game.get_card(card_id).map(|c| c.name.clone()).unwrap_or_default();
        let produced: Vec<String> = land_types.iter().map(|lt| format!("{{{}}}", lt.produces())).collect();
        println!("  Tapped {} for {}", name, produced.join(""));
        println!("  Mana pool: {}", game.player(player_id).mana_pool);
    }
}

fn play_land(game: &mut GameState, player_id: PlayerId) {
    if !game.player(player_id).can_play_land() {
        println!("  No land plays remaining this turn.");
        return;
    }

    let lands: Vec<(usize, CardId, String)> = game
        .player(player_id)
        .zones
        .hand
        .iter()
        .enumerate()
        .filter_map(|(i, &id)| {
            let card = game.get_card(id)?;
            if card.is_land() {
                Some((i, id, card.name.clone()))
            } else {
                None
            }
        })
        .collect();

    if lands.is_empty() {
        println!("  No lands in hand.");
        return;
    }

    println!("  Play which land?");
    for (idx, (_, _, name)) in lands.iter().enumerate() {
        println!("    [{}] {}", idx + 1, name);
    }
    println!("    [B] Back");

    if let Some(choice) = prompt_number("  > ", lands.len()) {
        let (_, card_id, name) = &lands[choice - 1];
        let card_id = *card_id;
        let name = name.clone();

        game.player_mut(player_id)
            .zones
            .move_card(card_id, Zone::Hand, Zone::Battlefield);

        let mut perm = PermanentState::new(card_id);
        perm.summoning_sick = false;
        game.permanents.insert(card_id, perm);
        game.player_mut(player_id).land_plays_remaining -= 1;

        println!("  Played {}", name);
    }
}

fn cast_spell(game: &mut GameState, player_id: PlayerId) {
    let castable: Vec<(usize, CardId, String)> = game
        .player(player_id)
        .zones
        .hand
        .iter()
        .enumerate()
        .filter_map(|(i, &id)| {
            let card = game.get_card(id)?;
            if !card.is_land() && game.player(player_id).mana_pool.can_pay(&card.cost) {
                Some((i, id, card.to_string()))
            } else {
                None
            }
        })
        .collect();

    if castable.is_empty() {
        println!("  No spells you can cast right now. (Tap lands for mana first?)");
        return;
    }

    println!("  Cast which spell?");
    for (idx, (_, _, display)) in castable.iter().enumerate() {
        println!("    [{}] {}", idx + 1, display);
    }
    println!("    [B] Back");

    if let Some(choice) = prompt_number("  > ", castable.len()) {
        let (_, card_id, _) = &castable[choice - 1];
        let card_id = *card_id;

        let cost = game.get_card(card_id).unwrap().cost.clone();
        let name = game.get_card(card_id).unwrap().name.clone();
        let is_creature = game.get_card(card_id).unwrap().is_creature();
        let has_haste = game.get_card(card_id).unwrap().has_keyword(Keyword::Haste);

        game.player_mut(player_id).mana_pool.pay(&cost);

        game.player_mut(player_id)
            .zones
            .move_card(card_id, Zone::Hand, Zone::Battlefield);

        let mut perm = PermanentState::new(card_id);
        if has_haste || !is_creature {
            perm.summoning_sick = false;
        }
        game.permanents.insert(card_id, perm);

        println!("  Cast {}!", name);
        if is_creature && !has_haste {
            println!("  {} enters the battlefield with summoning sickness.", name);
        }
        println!("  Mana pool: {}", game.player(player_id).mana_pool);
    }
}

fn declare_attackers(game: &mut GameState, active_player: PlayerId) -> bool {
    let eligible: Vec<(CardId, String)> = game
        .player(active_player)
        .zones
        .battlefield
        .iter()
        .filter_map(|&id| {
            let card = game.get_card(id)?;
            let perm = game.get_permanent(id)?;
            if card.is_creature()
                && perm.can_attack()
                && !card.has_keyword(Keyword::Defender)
            {
                let p = card.power.unwrap_or(0) + perm.power_modifier;
                let t = card.toughness.unwrap_or(0) + perm.toughness_modifier;
                Some((id, format!("{} {}/{}", card.name, p, t)))
            } else {
                None
            }
        })
        .collect();

    if eligible.is_empty() {
        println!("  No creatures can attack.");
        return false;
    }

    println!("  Declare attackers (enter numbers separated by spaces, or 'none'):");
    for (idx, (_, display)) in eligible.iter().enumerate() {
        println!("    [{}] {}", idx + 1, display);
    }

    let input = prompt("  > ");
    if input == "none" || input.is_empty() {
        println!("  No attackers declared.");
        return false;
    }

    let choices: Vec<usize> = input
        .split_whitespace()
        .filter_map(|s| s.parse::<usize>().ok())
        .filter(|&n| n >= 1 && n <= eligible.len())
        .collect();

    if choices.is_empty() {
        println!("  No valid attackers selected.");
        return false;
    }

    game.combat.clear();
    for &choice in &choices {
        let (card_id, _) = &eligible[choice - 1];
        let card_id = *card_id;
        game.combat.declare_attacker(card_id);

        let has_vigilance = game
            .get_card(card_id)
            .map(|c| c.has_keyword(Keyword::Vigilance))
            .unwrap_or(false);

        if !has_vigilance {
            if let Some(perm) = game.get_permanent_mut(card_id) {
                perm.tap();
            }
        }
    }

    let attacker_names: Vec<String> = choices
        .iter()
        .map(|&c| eligible[c - 1].1.clone())
        .collect();
    println!("  Attacking with: {}", attacker_names.join(", "));
    true
}

fn declare_blockers(game: &mut GameState, defending_player: PlayerId) {
    if game.combat.attackers.is_empty() {
        return;
    }

    let eligible_blockers: Vec<(CardId, String)> = game
        .player(defending_player)
        .zones
        .battlefield
        .iter()
        .filter_map(|&id| {
            let card = game.get_card(id)?;
            let perm = game.get_permanent(id)?;
            if card.is_creature() && !perm.tapped {
                let p = card.power.unwrap_or(0) + perm.power_modifier;
                let t = card.toughness.unwrap_or(0) + perm.toughness_modifier;
                Some((id, format!("{} {}/{}", card.name, p, t)))
            } else {
                None
            }
        })
        .collect();

    if eligible_blockers.is_empty() {
        println!("  {} has no creatures that can block.", game.player(defending_player).name);
        return;
    }

    println!();
    println!("  {}'s blockers:", game.player(defending_player).name);
    println!("  Attackers:");
    for (idx, attacker) in game.combat.attackers.iter().enumerate() {
        let name = game
            .get_card(attacker.card_id)
            .map(|c| c.to_string())
            .unwrap_or_default();
        println!("    [A{}] {}", idx + 1, name);
    }
    println!("  Available blockers:");
    for (idx, (_, display)) in eligible_blockers.iter().enumerate() {
        println!("    [{}] {}", idx + 1, display);
    }
    println!("  For each blocker, type: <blocker#> <attacker#> (e.g. '1 2' blocks attacker A2 with blocker 1)");
    println!("  Enter assignments one per line, empty line when done:");

    loop {
        let input = prompt("  block> ");
        if input.is_empty() {
            break;
        }

        let parts: Vec<usize> = input
            .split_whitespace()
            .filter_map(|s| s.parse::<usize>().ok())
            .collect();

        if parts.len() == 2 {
            let blocker_idx = parts[0];
            let attacker_idx = parts[1];

            if blocker_idx >= 1
                && blocker_idx <= eligible_blockers.len()
                && attacker_idx >= 1
                && attacker_idx <= game.combat.attackers.len()
            {
                let blocker_id = eligible_blockers[blocker_idx - 1].0;
                let attacker_id = game.combat.attackers[attacker_idx - 1].card_id;

                let attacker_has_flying = game
                    .get_card(attacker_id)
                    .map(|c| c.has_keyword(Keyword::Flying))
                    .unwrap_or(false);
                let blocker_can_block_flyer = game
                    .get_card(blocker_id)
                    .map(|c| c.has_keyword(Keyword::Flying) || c.has_keyword(Keyword::Reach))
                    .unwrap_or(false);

                if attacker_has_flying && !blocker_can_block_flyer {
                    println!("    Can't block — attacker has flying and blocker doesn't have flying or reach.");
                    continue;
                }

                game.combat.declare_blocker(blocker_id, attacker_id);
                println!(
                    "    {} blocks {}",
                    eligible_blockers[blocker_idx - 1].1,
                    game.get_card(attacker_id)
                        .map(|c| c.name.as_str())
                        .unwrap_or("???")
                );
            } else {
                println!("    Invalid numbers.");
            }
        } else {
            println!("    Format: <blocker#> <attacker#>");
        }
    }
}

fn resolve_combat_damage(game: &mut GameState, defending_player: PlayerId) {
    if game.combat.attackers.is_empty() {
        return;
    }

    println!("  --- Combat Damage ---");

    let attackers = game.combat.attackers.clone();

    for attacker_info in &attackers {
        let attacker_id = attacker_info.card_id;
        let attacker_card = match game.get_card(attacker_id) {
            Some(c) => c.clone(),
            None => continue,
        };
        let attacker_perm = match game.get_permanent(attacker_id) {
            Some(p) => p.clone(),
            None => continue,
        };

        let attacker_power = attacker_card.power.unwrap_or(0)
            + attacker_perm.power_modifier
            + attacker_perm.counters.plus_one
            - attacker_perm.counters.minus_one;

        if attacker_power <= 0 {
            continue;
        }

        let blockers = game.combat.get_blockers_for(attacker_id);

        if blockers.is_empty() {
            game.player_mut(defending_player).deal_damage(attacker_power);
            println!(
                "  {} deals {} damage to {} (life: {})",
                attacker_card.name,
                attacker_power,
                game.player(defending_player).name,
                game.player(defending_player).life,
            );

            if attacker_card.has_keyword(Keyword::Lifelink) {
                let controller = find_controller(game, attacker_id);
                game.player_mut(controller).gain_life(attacker_power);
                println!("    Lifelink: gained {} life", attacker_power);
            }
        } else {
            let mut remaining_power = attacker_power;
            let has_trample = attacker_card.has_keyword(Keyword::Trample);
            let has_deathtouch = attacker_card.has_keyword(Keyword::Deathtouch);

            for &blocker_id in &blockers {
                let blocker_card = match game.get_card(blocker_id) {
                    Some(c) => c.clone(),
                    None => continue,
                };
                let blocker_perm = match game.get_permanent(blocker_id) {
                    Some(p) => p.clone(),
                    None => continue,
                };

                let blocker_power = blocker_card.power.unwrap_or(0)
                    + blocker_perm.power_modifier
                    + blocker_perm.counters.plus_one
                    - blocker_perm.counters.minus_one;
                let blocker_toughness = blocker_card.toughness.unwrap_or(0)
                    + blocker_perm.toughness_modifier
                    + blocker_perm.counters.plus_one
                    - blocker_perm.counters.minus_one;

                let damage_to_blocker = if has_deathtouch {
                    1.min(remaining_power)
                } else {
                    (blocker_toughness - blocker_perm.damage_marked).max(0).min(remaining_power)
                };

                if let Some(perm) = game.get_permanent_mut(blocker_id) {
                    perm.damage_marked += damage_to_blocker;
                }
                remaining_power -= damage_to_blocker;

                println!(
                    "  {} deals {} damage to {}",
                    attacker_card.name, damage_to_blocker, blocker_card.name,
                );

                if blocker_power > 0 {
                    if let Some(perm) = game.get_permanent_mut(attacker_id) {
                        perm.damage_marked += blocker_power;
                    }
                    println!(
                        "  {} deals {} damage to {}",
                        blocker_card.name, blocker_power, attacker_card.name,
                    );
                }
            }

            if has_trample && remaining_power > 0 {
                game.player_mut(defending_player).deal_damage(remaining_power);
                println!(
                    "  {} tramples {} damage to {} (life: {})",
                    attacker_card.name,
                    remaining_power,
                    game.player(defending_player).name,
                    game.player(defending_player).life,
                );
            }
        }
    }

    game.check_state_based_actions();
    game.combat.clear();
}

fn find_controller(game: &GameState, card_id: CardId) -> PlayerId {
    for player in &game.players {
        if player.zones.battlefield.contains(&card_id) {
            return player.id;
        }
    }
    0
}

fn do_untap_step(game: &mut GameState) {
    let active = game.turn.active_player;
    let battlefield = game.player(active).zones.battlefield.clone();
    for card_id in &battlefield {
        if let Some(perm) = game.get_permanent_mut(*card_id) {
            perm.untap();
            if perm.summoning_sick {
                perm.summoning_sick = false;
            }
        }
    }
    game.player_mut(active).reset_for_turn();
    println!("  Untapped all permanents for {}.", game.player(active).name);
}

fn do_draw_step(game: &mut GameState) {
    let active = game.turn.active_player;
    if game.turn.turn_number == 1 && active == 0 {
        println!("  (First player skips draw on turn 1)");
        return;
    }
    if let Some(card_id) = game.player_mut(active).zones.draw_from_library() {
        let name = game
            .get_card(card_id)
            .map(|c| c.name.clone())
            .unwrap_or("???".into());
        println!("  {} drew: {}", game.player(active).name, name);
    } else {
        println!("  {} cannot draw — library empty!", game.player(active).name);
    }
}

fn do_cleanup(game: &mut GameState) {
    let active = game.turn.active_player;

    while game.player(active).hand_size() > 7 {
        display_hand(game, active);
        println!("  Must discard to 7 cards. Discard which?");
        if let Some(choice) = prompt_number("  > ", game.player(active).hand_size()) {
            let card_id = game.player(active).zones.hand[choice - 1];
            let name = game
                .get_card(card_id)
                .map(|c| c.name.clone())
                .unwrap_or_default();
            game.player_mut(active)
                .zones
                .move_card(card_id, Zone::Hand, Zone::Graveyard);
            println!("  Discarded {}", name);
        }
    }

    for player in &mut game.players {
        player.mana_pool.empty();
    }

    let all_perms: Vec<CardId> = game.permanents.keys().copied().collect();
    for card_id in all_perms {
        if let Some(perm) = game.get_permanent_mut(card_id) {
            perm.damage_marked = 0;
        }
    }
}

fn main_phase_actions(game: &mut GameState, player_id: PlayerId) {
    loop {
        display_game_state(game);
        display_hand(game, player_id);

        let can_play_land = game.player(player_id).can_play_land()
            && game
                .player(player_id)
                .zones
                .hand
                .iter()
                .any(|&id| game.get_card(id).map(|c| c.is_land()).unwrap_or(false));

        let has_untapped_lands = game
            .player(player_id)
            .zones
            .battlefield
            .iter()
            .any(|&id| {
                let is_land = game.get_card(id).map(|c| c.is_land()).unwrap_or(false);
                let untapped = game.get_permanent(id).map(|p| !p.tapped).unwrap_or(false);
                let has_types = game
                    .get_card(id)
                    .map(|c| !c.basic_land_types.is_empty())
                    .unwrap_or(false);
                is_land && untapped && has_types
            });

        let can_cast = game.player(player_id).zones.hand.iter().any(|&id| {
            let card = game.get_card(id);
            card.map(|c| !c.is_land() && game.player(player_id).mana_pool.can_pay(&c.cost))
                .unwrap_or(false)
        });

        println!();
        println!("  Actions:");
        if can_play_land {
            println!("    [L] Play a land");
        }
        if has_untapped_lands {
            println!("    [T] Tap a land for mana");
        }
        if can_cast {
            println!("    [C] Cast a spell");
        }
        println!("    [A] Go to combat");
        println!("    [P] Pass (end main phase)");
        println!("    [S] Show game state");
        println!("    [Q] Quit game");

        let input = prompt("  > ");
        match input.as_str() {
            "l" if can_play_land => play_land(game, player_id),
            "t" if has_untapped_lands => tap_land_for_mana(game, player_id),
            "c" if can_cast => cast_spell(game, player_id),
            "a" => return,
            "p" => return,
            "s" => display_game_state(game),
            "q" => {
                println!("  Thanks for playing!");
                std::process::exit(0);
            }
            _ => println!("  Invalid action."),
        }

        game.check_state_based_actions();
        if game.status != GameStatus::InProgress {
            return;
        }
    }
}

pub fn run_game(game: &mut GameState) {
    println!("╔══════════════════════════════════════╗");
    println!("║     MTG Engine — Modern Simulator    ║");
    println!("╚══════════════════════════════════════╝");
    println!();

    // Draw opening hands
    for player_id in 0..game.players.len() as u8 {
        for _ in 0..7 {
            game.player_mut(player_id).zones.draw_from_library();
        }
    }

    println!("Opening hands drawn.");
    for player_id in 0..game.players.len() as u8 {
        display_hand(game, player_id);
    }

    loop {
        if game.status != GameStatus::InProgress {
            break;
        }

        let active = game.turn.active_player;

        // === Untap Step ===
        game.turn.step = Step::Untap;
        display_game_state(game);
        do_untap_step(game);

        // === Upkeep ===
        game.turn.step = Step::Upkeep;

        // === Draw Step ===
        game.turn.step = Step::Draw;
        do_draw_step(game);

        // === Precombat Main Phase ===
        game.turn.step = Step::Main;
        game.turn.is_precombat_main = true;
        main_phase_actions(game, active);

        if game.status != GameStatus::InProgress {
            break;
        }

        // === Combat ===
        game.turn.step = Step::BeginCombat;
        let defending = (active + 1) % game.players.len() as u8;

        game.turn.step = Step::DeclareAttackers;
        display_game_state(game);
        let has_attackers = declare_attackers(game, active);

        if has_attackers {
            game.turn.step = Step::DeclareBlockers;
            display_game_state(game);
            declare_blockers(game, defending);

            game.turn.step = Step::CombatDamage;
            resolve_combat_damage(game, defending);

            if game.status != GameStatus::InProgress {
                break;
            }
        }

        game.turn.step = Step::EndOfCombat;

        // === Postcombat Main Phase ===
        game.turn.step = Step::Main;
        game.turn.is_precombat_main = false;
        main_phase_actions(game, active);

        if game.status != GameStatus::InProgress {
            break;
        }

        // === End Step ===
        game.turn.step = Step::EndStep;

        // === Cleanup ===
        game.turn.step = Step::Cleanup;
        do_cleanup(game);

        // === End Turn ===
        game.turn.end_turn(game.players.len() as u8);
    }

    // Game over
    println!();
    println!("══════════════════════════════════════════════════");
    match game.status {
        GameStatus::Won(id) => println!("  {} wins!", game.player(id).name),
        GameStatus::Draw => println!("  The game is a draw!"),
        _ => {}
    }
    display_game_state(game);
    println!("══════════════════════════════════════════════════");
}
