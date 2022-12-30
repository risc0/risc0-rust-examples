#[macro_use]
extern crate static_assertions;

pub mod merkle;

use std::ops::Range;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PrivateInput {
    /// Merkle tree root committing to a vector of data.
    pub root: merkle::Node,

    /// Width and height of the committed image.
    pub dimensions: (u32, u32),

    /// Range of indices to access to and verify subsequence membership.
    pub range: Range<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Journal {
    pub subsequence: Vec<[u8; 3]>,

    /// Merkle tree root of the committed image.
    /// Must be checked against the root of the image that was expected to be cropped.
    pub root: merkle::Node,

    /// Width and height of the committed image.
    /// Must be checked against the dimensions of the image that was expected to be cropped.
    pub dimensions: (u32, u32),
}
