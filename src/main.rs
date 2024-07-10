#![allow(warnings)]
use reinfors::{
    evaluator::MinimaxEvaluator,
    game_player::GamePlayer,
    game_state::{error::GameError, outcome::WinDraw},
    games::{
        masked_tic_tac_toe::{Info, MaskedEvaluator, MaskedTicTacToe},
        tic_tac_toe::{Piece, TicTacToe, ALL_ACTIONS},
    },
    strategy::GreedyStrategy,
};
fn main() {
    let masked = [ALL_ACTIONS[0], ALL_ACTIONS[1]];
    let state = MaskedTicTacToe::new(masked);
    let evaluator = MaskedEvaluator::new();
    let strategy = GreedyStrategy;
    let mut game_player = GamePlayer::new(state, evaluator, strategy);
    let (final_state, outcome) = game_player.play_interactive(true);
    match outcome {
        WinDraw::Win(player) => println!("{} wins!", player),
        WinDraw::Draw => println!("The game ended in a draw."),
    };
    //dbg!(final_state.history());
    //dbg!(game_player.evaluator());
    // let after0 = genesis.apply_unchecked(&ALL_ACTIONS[0]);
    // dbg!(after0.apply_unchecked(&ALL_ACTIONS[1]).visible_history());
    // dbg!(game_player.evaluator().evaluate(&genesis, &ALL_ACTIONS[0]));
    // let genesis = final_state.genesis();
    // for action in &ALL_ACTIONS {
    // dbg!(game_player.evaluator().evaluate(&genesis, &action));
    // }
}
