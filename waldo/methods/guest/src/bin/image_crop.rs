#![no_main]
// #![no_std]

use merkle::{Node, Proof, ShaHasher, ShaImpl};
use risc0_zkvm_guest::env;

risc0_zkvm_guest::entry!(main);

pub fn main() {
    // Read a merkle proof from the host.
    // TODO: Implement a nicer way for proofs to be serialized and deserialized.
    let lemma: Vec<Node> = env::read();
    let path: Vec<bool> = env::read();
    let value: u8 = env::read();
    let proof = Proof::new(lemma, path);

    assert!(proof.validate::<ShaHasher<ShaImpl>>());
    env::commit(&merkle_proof.path());
    env::commit(&merkle_proof.path());
}
