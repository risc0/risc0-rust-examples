#[macro_use]
extern crate static_assertions;

pub mod merkle;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PrivateInput {
    pub subsequence: Vec<u8>,

    /// Proofs is a vector of proofs that attest to inclusion of the subsequence into a single
    /// Merkle tree.
    // NOTE: This structure (and the subsequent verification) could be improved with support for
    // batch proofs that deduplicates the common nodes among the paths, and allows for batched
    // verification.
    pub proofs: Vec<merkle::Proof<u8>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Journal {
    pub subsequence: Vec<u8>,
    pub root: merkle::Node,
}
