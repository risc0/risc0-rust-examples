use std::io::prelude::*;

use methods::{SEARCH_JSON_ID, SEARCH_JSON_PATH};
use json_core::Outputs;
use risc0_zkvm::host::Prover;
use risc0_zkvm::serde::{from_slice, to_vec};

fn main() {
    let mut file =
        std::fs::File::open("res/example.json").expect("Example file should be accessible");
    let mut data = String::new();
    file.read_to_string(&mut data)
        .expect("Should not have I/O errors");

    // Make the prover.
    let method_code = std::fs::read(SEARCH_JSON_PATH).expect("Method code should be at path");
    let mut prover = Prover::new(&method_code, SEARCH_JSON_ID)
        .expect("Prover should be constructed from matching method code & ID");

    prover.add_input(&to_vec(&data).unwrap()).unwrap();

    // Run prover & generate receipt
    let receipt = prover.run().expect("Code should be provable");

    receipt
        .verify(SEARCH_JSON_ID)
        .expect("Proven code should verify");

    let journal = &receipt
        .get_journal_vec()
        .expect("Receipt should have journal");
    let outputs: Outputs = from_slice(&journal).expect("Journal should contain an Outputs object");

    println!("\nThe JSON file with hash\n  {}\nprovably contains a field 'critical_data' with value {}\n", outputs.hash, outputs.data);
}
