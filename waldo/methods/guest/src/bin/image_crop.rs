#![no_main]
// #![no_std]

use divrem::{DivCeil, DivRem};
use image::{imageops, GenericImageView, Rgb};
use risc0_zkvm_guest::env;
use waldo_core::merkle::{Node, VectorOracle};
use waldo_core::{Journal, PrivateInput};

risc0_zkvm_guest::entry!(main);

/// ImageOracle provides verified access to an image held by the host.
pub struct ImageOracle<const N: u32> {
    vector: VectorOracle<Vec<u8>>,
    width: u32,
    width_chunks: u32,
    height: u32,
}

impl<const N: u32> ImageOracle<N> {
    pub fn new(root: Node, width: u32, height: u32) -> Self {
        Self {
            vector: VectorOracle::<Vec<u8>>::new(root),
            width,
            width_chunks: width.div_ceil(N),
            height,
        }
    }

    pub fn root(&self) -> &Node {
        self.vector.root()
    }
}

impl<const N: u32> GenericImageView for ImageOracle<N> {
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

        let (x_chunk, x_offset) = x.div_rem(N);
        let (y_chunk, y_offset) = y.div_rem(N);

        let chunk = self
            .vector
            .get(usize::try_from(y_chunk * self.width_chunks + x_chunk).unwrap());

        // DO NOT MERGE: Does not handle chunks on edges, where the width and heigh may not be N.
        <[u8; 3]>::try_from(&chunk[usize::try_from((y_offset * N + x_offset) * 3).unwrap()..][..3])
            .unwrap()
            .into()
    }
}

pub fn main() {
    // Read a Merkle proof from the host.
    let input: PrivateInput = env::read();

    // Initialize a Merkle tree based vector oracle, supporting verified access to a vector of data
    // on the host. Use the oracle to access a range of elements from the host.
    let oracle = ImageOracle::<8>::new(
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
