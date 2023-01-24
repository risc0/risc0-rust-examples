#![no_main]

use shakmaty::{Chess, Position, fen::Fen, Setup, CastlingMode, FromSetup, Move, san::San};
use chess_core::Inputs;
use risc0_zkvm::guest::env;

risc0_zkvm::guest::entry!(main);

pub fn main() {
    let inputs: Inputs = env::read();
    let mv: String = inputs.mv;
    let initial_state: String = inputs.board;
    env::commit(&initial_state);

    let setup = Setup::from(Fen::from_ascii(initial_state.as_bytes()).unwrap());
    let pos = Chess::from_setup(setup, CastlingMode::Standard).unwrap();

    let mv: Move = mv.parse::<San>().unwrap().to_move(&pos).unwrap();
    let pos = pos.play(&mv).unwrap();
    assert!(pos.is_checkmate());
}
