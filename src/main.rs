use reinfors::{
    evaluator::EndStateEvaluator,
    game_player::GamePlayer,
    game_state::WinDrawOutcome,
    games::tic_tac_toe::{BoardState, Piece},
    strategy::MinMax,
};
fn main() {
    let state = BoardState::new(Piece::X);
    let strategy = MinMax;
    let evaluator = EndStateEvaluator::new();
    let mut player = GamePlayer::new(state, evaluator, strategy);
    match player.play_interactive(1) {
        Ok(WinDrawOutcome::Win(player)) => println!("{} wins!", player),
        Ok(WinDrawOutcome::Draw) => println!("The game ended in a draw."),
        Err(err) => println!("{:?}", err),
    }
}
