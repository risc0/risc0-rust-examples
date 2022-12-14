#![no_main]
// #![no_std]

use merkle_light::hash::{Algorithm, Hashable};
use risc0_zkvm_guest::env;
use waldo_core::{Node, Proof, ShaHasher, ShaImpl};

risc0_zkvm_guest::entry!(main);

/// Verify a Merkle proof opening for a given value is consistent with the committed root.
fn verify_merkle_opening(root: &Node, proof: &Proof, value: u8) -> bool {
    // Check that the root of the proof matches the commitment.
    if &proof.root() != root {
        return false;
    }

    // Check that the path from the leaf matches the root.
    if !proof.validate::<ShaHasher<ShaImpl>>() {
        return false;
    }

    // Check the value hashes to the leaf in the proof.
    // Hash the value.
    let algorithm = &mut ShaHasher::<ShaImpl>::default();
    value.hash(algorithm);
    let value_hash = algorithm.hash();

    // Hash the hash of the  value to get the leaf.
    algorithm.reset();
    let leaf_hash = algorithm.leaf(value_hash);

    leaf_hash == proof.item()
}

pub fn main() {
    // Read a Merkle proof from the host.
    // TODO: Implement a nicer way for proofs to be serialized and deserialized.
    let lemma: Vec<Node> = env::read();
    let path: Vec<bool> = env::read();
    let value: u8 = env::read();
    let proof = Proof::new(lemma, path);
    let root = proof.root();

    // TODO: Verify that the claimed value is actually the value committed in the proof.
    assert!(verify_merkle_opening(&root, &proof, value));

    env::commit(&root);
    env::commit(&value);
}
