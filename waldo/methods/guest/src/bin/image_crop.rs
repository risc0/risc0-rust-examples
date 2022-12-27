#![no_main]
// #![no_std]

use risc0_zkvm_guest::env;
use waldo_core::{Journal, PrivateInput};

risc0_zkvm_guest::entry!(main);

pub fn main() {
    // Read a Merkle proof from the host.
    let input: PrivateInput = env::read();
    let root = input.proofs[0].root();
    let value = input.subsequence[0];

    assert!(input.proofs[0].verify(&root, value));

    let journal = Journal {
        subsequence: vec![value],
        root: root,
    };
    env::commit(&journal);
}
