#[macro_use]
extern crate static_assertions;

pub mod merkle;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PrivateInput {
    subsequence: Vec<u8>,
    proofs: Vec<merkle::Proof<u8>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Journal {
    subsequence: Vec<u8>,
    root: merkle::Node,
}
