#![no_main]
// #![no_std]

use image::{imageops, GenericImageView};
use risc0_zkvm::guest::env;
use waldo_core::image::{ImageMask, ImageOracle, IMAGE_CHUNK_SIZE};
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

    let subimage = imageops::crop_imm(
        &oracle,
        input.crop_location.0,
        input.crop_location.1,
        input.crop_dimensions.0,
        input.crop_dimensions.1,
    )
    .to_image();

    // If a mask is provided, apply it to reveal less about the image.
    let subimage_masked = match input.mask {
        Some(mask_raw) => {
            let mask =
                ImageMask::from_raw(input.crop_dimensions.0, input.crop_dimensions.1, mask_raw)
                    .unwrap();
            mask.apply(subimage)
        }
        None => subimage,
    };

    // Collect the verified public information into the journal.
    let journal = Journal {
        root: *oracle.root(),
        image_dimensions: oracle.dimensions(),
        subimage_dimensions: subimage_masked.dimensions(),
        subimage: subimage_masked.into_raw(),
    };
    env::commit(&journal);
}
