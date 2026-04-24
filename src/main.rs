use crate::game::Game;

use crate::game::DecisionSelector;
use std::rc::Rc;
mod game;
mod probabilistic;
mod xo;

fn main() {
    env_logger::init();

    let decision_selector: Rc<dyn DecisionSelector> = Rc::new(game::CliDecisionSelector {});

    let mut game = xo::XOGame::new();
    let o_player = xo::XOPlayer::new(Rc::clone(&decision_selector));
    let x_player = xo::XOPlayer::new(Rc::clone(&decision_selector));

    game.join_player(xo::XOPlayerRole::X, o_player);
    game.join_player(xo::XOPlayerRole::O, x_player);

    game.step_all_forward();
}
