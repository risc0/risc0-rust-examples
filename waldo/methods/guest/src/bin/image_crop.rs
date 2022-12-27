#![no_main]
// #![no_std]

use risc0_zkvm::serde;
use risc0_zkvm_guest::env;
use waldo_core::merkle::Proof;
use waldo_core::{Journal, PrivateInput, VECTOR_ORACLE_CHANNEL};

risc0_zkvm_guest::entry!(main);

pub fn main() {
    // Read a Merkle proof from the host.
    let input: PrivateInput = env::read();

    let mut subsequence = Vec::<u8>::with_capacity(input.range.len());
    for i in input.range.map(|i| usize::try_from(i).unwrap()) {
        // NOTE: This loop  could be improved with support for batch proofs that deduplicates the
        // common nodes among the paths, and allows for batched verification.

        // Fetch the value and proof from the host by index.
        // NOTE: It would be nice if there was a wrapper for send_recv that looked more like
        // env::read(). A smaller step would be to have this method as take [u32] instead of [u8]
        // to avoid mucking around with the bytes.
        // TODO: Consider using bincode or another byte serializer instead of the u32 RISC0 format.
        let (value, proof): (u8, Proof<u8>) = serde::from_slice(
            env::send_recv_as_u32(
                VECTOR_ORACLE_CHANNEL,
                bytemuck::cast_slice(&serde::to_vec(&(i as u32)).unwrap()),
            )
            .0,
        )
        .unwrap();

        // Verify the proof.
        assert_eq!(i, proof.index());
        assert!(proof.verify(&input.root, value));

        subsequence.push(value);
    }

    // Collect the verified public information into the journal.
    let journal = Journal {
        subsequence,
        root: input.root,
    };
    env::commit(&journal);
}
