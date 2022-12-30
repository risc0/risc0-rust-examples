#![no_main]
// #![no_std]

use image::{imageops, GenericImageView, Rgb};
use risc0_zkvm_guest::env;
use waldo_core::merkle::{Node, VectorOracle};
use waldo_core::{Journal, PrivateInput};

risc0_zkvm_guest::entry!(main);

/// ImageOracle provides verified access to an image held by the host.
// NOTE: ImageOracle accesses, and verifies via Merkel proofs, parts of an image on a per-pixel basis.
// This is likely to be highly innefficient compared to an approach that supports access on a less
// granular basis (e.g. by chunking the image which the Merkle tree elements cooresponding to row
// or square of the image)
pub struct ImageOracle {
    vector: VectorOracle<[u8; 3]>,
    width: u32,
    height: u32,
}

impl ImageOracle {
    pub fn new(root: Node, width: u32, height: u32) -> Self {
        Self {
            vector: VectorOracle::<[u8; 3]>::new(root),
            width,
            height,
        }
    }

    pub fn root(&self) -> &Node {
        self.vector.root()
    }
}

impl GenericImageView for ImageOracle {
    type Pixel = Rgb<u8>;

    fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn bounds(&self) -> (u32, u32, u32, u32) {
        (0, 0, self.width, self.height)
    }

    fn get_pixel(&self, x: u32, y: u32) -> Self::Pixel {
        if x >= self.width || y >= self.height {
            panic!(
                "out of bound image access: ({}, {}) on {}x{} image",
                x, y, self.width, self.height
            );
        }

        self.vector
            .get(usize::try_from(y * self.width + x).unwrap())
            .into()
    }
}

pub fn main() {
    // Read a Merkle proof from the host.
    let input: PrivateInput = env::read();

    // Initialize a Merkle tree based vector oracle, supporting verified access to a vector of data
    // on the host. Use the oracle to access a range of elements from the host.
    let oracle = ImageOracle::new(
        input.root,
        input.image_dimensions.0,
        input.image_dimensions.1,
    );

    let crop = imageops::crop_imm(
        &oracle,
        input.crop_locaction.0,
        input.crop_locaction.1,
        input.crop_dimensions.0,
        input.crop_dimensions.1,
    );

    // Collect the verified public information into the journal.
    let journal = Journal {
        subsequence: crop
            .pixels()
            .map(|(_, _, pixel): (_, _, Rgb<u8>)| pixel.0)
            .collect(),
        root: *oracle.root(),
        image_dimensions: oracle.dimensions(),
    };
    env::commit(&journal);
}
