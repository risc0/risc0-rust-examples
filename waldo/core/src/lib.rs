#[macro_use]
extern crate static_assertions;

pub mod merkle;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PrivateInput {
    /// Merkle tree root committing to a vector of data.
    pub root: merkle::Node,

    /// Width and height of the committed image.
    pub image_dimensions: (u32, u32),

    /// X and y location for the top left corner of the crop.
    pub crop_locaction: (u32, u32),

    /// X and y location for the top left corner of the crop.
    pub crop_dimensions: (u32, u32),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Journal {
    pub subsequence: Vec<[u8; 3]>,

    /// Merkle tree root of the committed image.
    /// Must be checked against the root of the image that was expected to be cropped.
    pub root: merkle::Node,

    /// Width and height of the committed image.
    /// Must be checked against the dimensions of the image that was expected to be cropped.
    pub image_dimensions: (u32, u32),
}
