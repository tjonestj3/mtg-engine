use mtg_engine::data;
use mtg_engine::game::game_loop;
use mtg_engine::game::state::GameState;

fn main() {
    let mut game = GameState::new(vec!["Player 1".into(), "Player 2".into()]);

    for player_id in 0..2u8 {
        let deck = data::sample_decklist();
        for card_data in deck {
            let card_id = game.register_card(card_data);
            game.player_mut(player_id).zones.library.push(card_id);
        }
        game.player_mut(player_id).zones.shuffle_library();
    }

    game_loop::run_game(&mut game);
}
