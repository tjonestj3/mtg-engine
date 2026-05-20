use macroquad::prelude::*;

use mtg_engine::data;
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
    let mut game = GameState::new(vec!["Player 1".into(), "Player 2".into()]);

    for player_id in 0..2u8 {
        let deck = data::sample_decklist();
        for card_data in deck {
            let card_id = game.register_card(card_data);
            game.player_mut(player_id).zones.library.push(card_id);
        }
        game.player_mut(player_id).zones.shuffle_library();
    }

    ui::run_game_ui(&mut game).await;
}
