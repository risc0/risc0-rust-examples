use std::io::prelude::*;

use methods::{SEARCH_JSON_ID, SEARCH_JSON_PATH};
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

    // TODO: Implement code for transmitting or serializing the receipt for other parties to verify here
    let val: u32 = from_slice(
        &receipt
            .get_journal_vec()
            .expect("Journal should be available for valid receipts"),
    )
    .expect("Journal output should deserialize into the same types (& order) that it was written");

    let mut first_run: bool = true;
    let mut sha_str = String::from("");
    for entry in &receipt.get_journal_vec().expect("WIP") {
        if first_run {
            first_run = false;
        } else {
            sha_str += &format!("{entry:x}")[..];
        }
    }

    println!("\nThe JSON file with hash\n  {}\nprovably contains a field 'critical_data' with value {}\n", sha_str, val);
}
