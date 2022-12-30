#[macro_use]
extern crate static_assertions;

pub mod merkle;

use std::ops::Range;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PrivateInput {
    /// Merkle tree root committing to a vector of data.
    pub root: merkle::Node,

    /// Range of indices to access to and verify subsequence membership.
    pub range: Range<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Journal {
    pub subsequence: Vec<u8>,
    pub root: merkle::Node,
}
