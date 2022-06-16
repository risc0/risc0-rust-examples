use rand::prelude::*;

use std::fs;

use password_validity_checker_methods::{PW_CHECKER_ID, PW_CHECKER_PATH};                    
use risc0_zkvm_core::Digest;
use risc0_zkvm_host::Prover;
use risc0_zkvm_serde::{from_slice, to_vec};
use tempfile::tempdir;

fn main() {
    let password: &str = "S00perSecr1t!!!";

    let mut rng = StdRng::from_entropy();
    let mut salt_bytes = [0u8, 32];
    rng.fill(&mut salt_bytes);

    // Write the ID to a file.
    // This is to work around the fact that the C++ prover API doesn't take IDs as buffers currently.
    let temp_dir = tempdir().unwrap();
    let id_path = temp_dir
        .path()
        .join("pw_checker.id")
        .to_str()
        .unwrap()
        .to_string();
    fs::write(id_path, PW_CHECKER_ID).unwrap();

    // Using the file ID from the above workaround,
    // a new prover is created to run the pw_checker method
    let mut prover = Prover::new(&PW_CHECKER_PATH, PW_CHECKER_ID).unwrap();

    // Adding input to the prover makes it readable by the guest
    prover
        .add_input(to_vec(&password).unwrap().as_slice())
        .unwrap();
    prover
        .add_input(to_vec(&salt_bytes).unwrap().as_slice())
        .unwrap();

    let receipt = prover.run().unwrap();

    // Because the password hash has been committed to the journal,
    // we can read its value from the receipt
    let pw_hash: Digest = from_slice(&receipt.get_journal_vec().unwrap()).unwrap();
    println!("This is the hash of a valid password: {}", pw_hash);

    // Here is where one would send 'receipt' over the network...

    // Verify receipt, panic if it's wrong
    receipt.verify(PW_CHECKER_ID).unwrap();
}
