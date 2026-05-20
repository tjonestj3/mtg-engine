use macroquad::prelude::*;

use mtg_engine::data;
use mtg_engine::data::decklists;
use mtg_engine::data::scryfall;
use mtg_engine::game::state::GameState;
use mtg_engine::ui;

fn window_conf() -> Conf {
    Conf {
        window_title: "MTG Engine".to_owned(),
        window_width: 1280,
        window_height: 800,
        window_resizable: false,
        high_dpi: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // Deck selection screen
    let deck_choice = deck_select_screen().await;

    let mut game = GameState::new(vec!["Player 1".into(), "Player 2".into()]);

    match deck_choice {
        DeckChoice::SampleVsSample => {
            for pid in 0..2u8 {
                let deck = data::sample_decklist();
                for card_data in deck {
                    let id = game.register_card(card_data);
                    game.player_mut(pid).zones.library.push(id);
                }
                game.player_mut(pid).zones.shuffle_library();
            }
        }
        DeckChoice::TwinVsTitan => {
            load_scryfall_deck(&mut game, 0, &decklists::splinter_twin()).await;
            load_scryfall_deck(&mut game, 1, &decklists::amulet_titan()).await;
        }
        DeckChoice::TwinVsTwin => {
            load_scryfall_deck(&mut game, 0, &decklists::splinter_twin()).await;
            load_scryfall_deck(&mut game, 1, &decklists::splinter_twin()).await;
        }
        DeckChoice::TitanVsTitan => {
            load_scryfall_deck(&mut game, 0, &decklists::amulet_titan()).await;
            load_scryfall_deck(&mut game, 1, &decklists::amulet_titan()).await;
        }
    }

    ui::run_game_ui(&mut game).await;
}

#[derive(Clone, Copy)]
enum DeckChoice {
    SampleVsSample,
    TwinVsTitan,
    TwinVsTwin,
    TitanVsTitan,
}

async fn deck_select_screen() -> DeckChoice {
    loop {
        clear_background(Color::new(0.08, 0.08, 0.12, 1.0));

        draw_text("MTG Engine", 490.0, 120.0, 48.0, WHITE);
        draw_text("Select Matchup", 520.0, 180.0, 28.0, Color::new(0.6, 0.6, 0.6, 1.0));

        let choices = [
            ("1. Splinter Twin vs Amulet Titan", DeckChoice::TwinVsTitan),
            ("2. Twin vs Twin", DeckChoice::TwinVsTwin),
            ("3. Titan vs Titan", DeckChoice::TitanVsTitan),
            ("4. Sample Decks (no internet needed)", DeckChoice::SampleVsSample),
        ];

        let mouse = mouse_position();
        for (i, (label, _)) in choices.iter().enumerate() {
            let y = 260.0 + i as f32 * 60.0;
            let hovered = mouse.0 >= 350.0 && mouse.0 <= 930.0 && mouse.1 >= y - 30.0 && mouse.1 <= y + 10.0;

            let bg = if hovered {
                Color::new(0.2, 0.2, 0.35, 1.0)
            } else {
                Color::new(0.12, 0.12, 0.18, 1.0)
            };
            draw_rectangle(350.0, y - 30.0, 580.0, 45.0, bg);
            draw_rectangle_lines(350.0, y - 30.0, 580.0, 45.0, 1.0, Color::new(0.25, 0.25, 0.35, 1.0));
            draw_text(label, 370.0, y, 24.0, if hovered { WHITE } else { Color::new(0.8, 0.8, 0.8, 1.0) });

            if hovered && is_mouse_button_pressed(MouseButton::Left) {
                return choices[i].1;
            }
        }

        if is_key_pressed(KeyCode::Key1) { return DeckChoice::TwinVsTitan; }
        if is_key_pressed(KeyCode::Key2) { return DeckChoice::TwinVsTwin; }
        if is_key_pressed(KeyCode::Key3) { return DeckChoice::TitanVsTitan; }
        if is_key_pressed(KeyCode::Key4) { return DeckChoice::SampleVsSample; }

        draw_text(
            "Cards loaded from Scryfall API (cached after first fetch)",
            360.0, 560.0, 16.0, Color::new(0.5, 0.5, 0.5, 1.0),
        );

        next_frame().await;
    }
}

async fn load_scryfall_deck(game: &mut GameState, player_id: u8, entries: &[(u8, &str)]) {
    // Show loading screen
    clear_background(Color::new(0.08, 0.08, 0.12, 1.0));
    draw_text(
        &format!("Loading Player {}'s deck...", player_id + 1),
        450.0, 400.0, 28.0, WHITE,
    );
    next_frame().await;

    let deck = scryfall::fetch_decklist(entries, Some(&|msg| {
        println!("{}", msg);
    }));

    if deck.is_empty() {
        eprintln!("Warning: deck loaded 0 cards for player {}, falling back to sample", player_id + 1);
        let fallback = data::sample_decklist();
        for card_data in fallback {
            let id = game.register_card(card_data);
            game.player_mut(player_id).zones.library.push(id);
        }
    } else {
        println!("Player {} deck: {} cards loaded", player_id + 1, deck.len());
        for card_data in deck {
            let id = game.register_card(card_data);
            game.player_mut(player_id).zones.library.push(id);
        }
    }
    game.player_mut(player_id).zones.shuffle_library();
}
