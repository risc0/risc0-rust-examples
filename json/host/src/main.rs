use std::io::prelude::*;

use methods::{SEARCH_JSON_ID, SEARCH_JSON_PATH};
use risc0_zkp::core::sha::Digest;
use risc0_zkvm::host::Prover;
use risc0_zkvm::serde::{from_slice, to_vec};

fn main() {
    let mut file = std::fs::File::open("res/example.json").expect("Example file should be accessible");
    let mut data = String::new();
    file.read_to_string(&mut data).expect("Should not have I/O errors");

    // Make the prover.
    let method_code = std::fs::read(SEARCH_JSON_PATH)
        .expect("Method code should be present at the specified path; did you use the correct *_PATH constant?");
    let mut prover = Prover::new(&method_code, SEARCH_JSON_ID)
        .expect("Prover should be constructed from valid method source code and corresponding method ID");

    prover.add_input(&to_vec(&data).unwrap()).unwrap();

    // Run prover & generate receipt
    let receipt = prover.run()
        .expect("Valid code should be provable if it doesn't overflow the cycle limit. See `embed_methods_with_options` for information on adjusting maximum cycle count.");

    // Optional: Verify receipt to confirm that recipients will also be able to verify your receipt
    receipt.verify(SEARCH_JSON_ID)
        .expect("Code you have proven should successfully verify; did you specify the correct method ID?");

    let journal = &receipt.get_journal_vec().expect("Receipt should have journal");
    let val: u32 = journal[0];
    let digest = from_slice::<Digest>(&journal[1..]).expect("Journal should contain SHA Digest");

    println!("\nThe JSON file with hash\n  {}\nprovably contains a field 'critical_data' with value {}\n", digest, val);
}
