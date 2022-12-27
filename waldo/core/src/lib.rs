#[macro_use]
extern crate static_assertions;

pub mod merkle;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PrivateInput {
    pub subsequence: Vec<u8>,
    pub proofs: Vec<merkle::Proof<u8>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Journal {
    pub subsequence: Vec<u8>,
    pub root: merkle::Node,
}
