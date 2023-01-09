#![no_main]
// #![no_std]

use image::{imageops, GenericImageView};
use risc0_zkvm::guest::env;
use waldo_core::image::{ImageOracle, IMAGE_CHUNK_SIZE};
use waldo_core::{Journal, PrivateInput};

risc0_zkvm::guest::entry!(main);

pub fn main() {
    // Read a Merkle proof from the host.
    let input: PrivateInput = env::read();

    // Initialize a Merkle tree based vector oracle, supporting verified access to a vector of data
    // on the host. Use the oracle to access a range of elements from the host.
    let oracle = ImageOracle::<{ IMAGE_CHUNK_SIZE }>::new(
        input.root,
        input.image_dimensions.0,
        input.image_dimensions.1,
    );

    let crop = imageops::crop_imm(
        &oracle,
        input.crop_location.0,
        input.crop_location.1,
        input.crop_dimensions.0,
        input.crop_dimensions.1,
    );

    // Collect the verified public information into the journal.
    let journal = Journal {
        root: *oracle.root(),
        image_dimensions: oracle.dimensions(),
        subimage: crop.to_image().into_raw(),
        subimage_dimensions: input.crop_dimensions,
    };
    env::commit(&journal);
}
