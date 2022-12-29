#![no_main]
// #![no_std]

use risc0_zkvm_guest::env;
use waldo_core::merkle::VectorOracle;
use waldo_core::{Journal, PrivateInput};

risc0_zkvm_guest::entry!(main);

pub fn main() {
    // Read a Merkle proof from the host.
    let input: PrivateInput = env::read();

    let oracle = VectorOracle::<u8>::new(input.root);
    let subsequence: Vec<_> = input
        .range
        .map(|i| usize::try_from(i).unwrap())
        .map(|i| oracle.get(i))
        .collect();

    // Collect the verified public information into the journal.
    let journal = Journal {
        subsequence,
        root: oracle.root,
    };
    env::commit(&journal);
}
