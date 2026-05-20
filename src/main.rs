use mtg_engine::data;
use mtg_engine::effect::PermanentState;
use mtg_engine::game::state::GameState;
use mtg_engine::zone::Zone;

fn main() {
    println!("╔══════════════════════════════════════╗");
    println!("║     MTG Engine — Modern Simulator    ║");
    println!("╚══════════════════════════════════════╝");
    println!();

    let mut game = GameState::new(vec!["Player 1".into(), "Player 2".into()]);

    // Load sample decks for both players
    for player_id in 0..2u8 {
        let deck = data::sample_decklist();
        for card_data in deck {
            let card_id = game.register_card(card_data);
            game.player_mut(player_id).zones.library.push(card_id);
        }
        game.player_mut(player_id).zones.shuffle_library();
    }

    // Draw opening hands (7 cards each)
    for player_id in 0..2u8 {
        for _ in 0..7 {
            game.player_mut(player_id).zones.draw_from_library();
        }
    }

    println!("{}", game);

    // Show each player's opening hand
    for player_id in 0..2u8 {
        let hand: Vec<String> = game
            .player(player_id)
            .zones
            .hand
            .iter()
            .filter_map(|id| game.get_card(*id))
            .map(|c| c.to_string())
            .collect();
        println!("{}'s hand:", game.player(player_id).name);
        for (i, card) in hand.iter().enumerate() {
            println!("  [{}] {}", i + 1, card);
        }
        println!();
    }

    // Demo: Player 1 plays a Forest and taps it for mana
    let p1_hand = game.players[0].zones.hand.clone();
    let forest_id = p1_hand.iter().find(|&&id| {
        game.get_card(id)
            .map(|c| c.name == "Forest")
            .unwrap_or(false)
    });

    if let Some(&forest_id) = forest_id {
        println!("--- Player 1 plays a Forest ---");
        game.players[0].zones.move_card(forest_id, Zone::Hand, Zone::Battlefield);
        game.permanents.insert(forest_id, PermanentState::new(forest_id));
        game.players[0].land_plays_remaining -= 1;

        // Tap for mana
        if let Some(perm) = game.get_permanent_mut(forest_id) {
            perm.tap();
        }
        let land_types = game
            .get_card(forest_id)
            .map(|c| c.basic_land_types.clone())
            .unwrap_or_default();
        for lt in &land_types {
            game.players[0].mana_pool.add(lt.produces(), 1);
        }
        println!("  Tapped Forest for {{G}}");
        println!("  Mana pool: {}", game.players[0].mana_pool);
        println!();

        // Cast a Llanowar Elves if in hand
        let elves_id = game.players[0].zones.hand.iter().find(|&&id| {
            game.get_card(id)
                .map(|c| c.name == "Llanowar Elves")
                .unwrap_or(false)
        }).copied();

        if let Some(elves_id) = elves_id {
            let cost = game.get_card(elves_id).unwrap().cost.clone();
            if game.players[0].mana_pool.pay(&cost) {
                println!("--- Player 1 casts Llanowar Elves ---");
                game.players[0].zones.move_card(elves_id, Zone::Hand, Zone::Battlefield);
                let mut perm = PermanentState::new(elves_id);
                perm.summoning_sick = true;
                game.permanents.insert(elves_id, perm);
                println!("  Llanowar Elves enters the battlefield (summoning sick)");
                println!("  Mana pool: {}", game.players[0].mana_pool);
            }
        }
    }

    println!();
    println!("{}", game);
    println!("Engine scaffolding complete. Ready to build out turn loop and rule enforcement!");
}
