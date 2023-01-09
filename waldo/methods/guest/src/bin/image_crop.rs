#![no_main]
// #![no_std]

use image::{imageops, GenericImageView, GrayImage, Luma, Rgb, RgbImage};
use risc0_zkvm::guest::env;
use waldo_core::image::{ImageOracle, IMAGE_CHUNK_SIZE};
use waldo_core::{Journal, PrivateInput};

risc0_zkvm::guest::entry!(main);

fn apply_image_mask(mut image: RgbImage, mask: &GrayImage) -> RgbImage {
    assert_eq!(image.dimensions(), mask.dimensions());

    for x in 0..image.width() {
        for y in 0..image.height() {
            let p: Rgb<u8> = *image.get_pixel(x, y);
            let m: Luma<u8> = *mask.get_pixel(x, y);
            image.put_pixel(
                x,
                y,
                [
                    p.0[0].saturating_sub(m.0[0]),
                    p.0[1].saturating_sub(m.0[0]),
                    p.0[2].saturating_sub(m.0[0]),
                ]
                .into(),
            );
        }
    }
    image
}

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

    // If provided, apply the provided mask to reveal less about the image.
    let subimage_masked = match input.mask {
        Some(mask_raw) => {
            let mask =
                GrayImage::from_raw(input.crop_dimensions.0, input.crop_dimensions.1, mask_raw)
                    .unwrap();
            apply_image_mask(subimage, &mask)
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
