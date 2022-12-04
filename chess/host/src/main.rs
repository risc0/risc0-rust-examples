use clap::{Arg, Command};
use methods::{CHECKMATE_ID, CHECKMATE_PATH};
use risc0_zkvm::host::Prover;
use risc0_zkvm::serde::{from_slice, to_vec};
use shakmaty::{fen::Fen, CastlingMode, Chess, FromSetup, Position, Setup};

use chess_core::Inputs;

fn pad_to_word(inp: &mut String) {
    inp.push_str(&"     "[..4 - (inp.len() % 4)]);
}

fn main() {
    let matches =
        Command::new("chess")
            .arg(Arg::new("move").default_value("Qxf7"))
            .arg(Arg::new("board").default_value(
                "r1bqkb1r/pppp1ppp/2n2n2/4p2Q/2B1P3/8/PPPP1PPP/RNB1K1NR w KQkq - 4 4",
            ))
            .get_matches();
    let mv = matches.get_one::<String>("move").unwrap();
    let mut initial_state = matches.get_one::<String>("board").unwrap().to_string();
    pad_to_word(&mut initial_state);

    let inputs = Inputs {
        board: initial_state,
        mv: mv.to_string(),
    };

    // Make the prover.
    let method_code = std::fs::read(CHECKMATE_PATH)
        .expect("Method code should be present at the specified path; did you use the correct *_PATH constant?");
    let mut prover = Prover::new(&method_code, CHECKMATE_ID).expect(
        "Prover should be constructed from valid method source code and corresponding method ID",
    );

    prover
        .add_input(&to_vec(&inputs).expect("input string should serialize"))
        .expect("Prover should accept input");

    // Run prover & generate receipt
    let receipt = prover.run()
        .expect("Valid code should be provable if it doesn't overflow the cycle limit. See `embed_methods_with_options` for information on adjusting maximum cycle count.");

    // Verify receipt and parse it for committed data
    receipt.verify(CHECKMATE_ID).expect(
        "Code you have proven should successfully verify; did you specify the correct method ID?",
    );
    let vec = receipt
        .get_journal_vec()
        .expect("Journal should be accessible");
    let committed_state: String = from_slice(&vec).expect("Journal should contain a FEN String");
    assert_eq!(inputs.board, committed_state);
    let fen = Fen::from_ascii(committed_state.as_bytes()).unwrap();
    let setup = Setup::from(fen);
    let pos = Chess::from_setup(setup, CastlingMode::Standard).unwrap();

    println!(
        "There is a checkmate for {:?} in this position:\n{:?}",
        pos.turn(),
        pos.board()
    );
}
