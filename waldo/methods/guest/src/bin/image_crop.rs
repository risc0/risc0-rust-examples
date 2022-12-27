#![no_main]
// #![no_std]

use risc0_zkvm_guest::env;
use waldo_core::{Journal, PrivateInput};

risc0_zkvm_guest::entry!(main);

pub fn main() {
    // Read a Merkle proof from the host.
    let input: PrivateInput = env::read();

    // Verify that the subsequence has at least one value, and the subquence length matches the
    // number of proofs. (Trivial case of inclusion of an empty subsequence is meaningless.)
    assert!(input.subsequence.len() > 0);
    assert!(input.subsequence.len() == input.proofs.len());

    // Pin the Merkle tree root to the first root in the sequence of proofs, then check all proofs
    // in the subsequence against the root, and check for adjacency.
    let root = input.proofs[0].root();
    let start_index = input.proofs[0].index();
    for (offset, (value, proof)) in input
        .subsequence
        .iter()
        .copied()
        .zip(input.proofs)
        .enumerate()
    {
        assert!(proof.verify(&root, value));
        assert_eq!(start_index + offset, proof.index());
    }

    let journal = Journal {
        subsequence: input.subsequence,
        root: root,
    };
    env::commit(&journal);
}
