use std::error::Error;

use image::io::Reader as ImageReader;
use image::{GenericImageView, Rgb};
use risc0_zkvm::host::{Prover, ProverOpts};
use risc0_zkvm::serde;
use waldo_core::merkle::{MerkleTree, VECTOR_ORACLE_CHANNEL};
use waldo_core::{Journal, PrivateInput};
use waldo_methods::{IMAGE_CROP_ID, IMAGE_CROP_PATH};

fn main() -> Result<(), Box<dyn Error>> {
    // Read the image from disk.
    let image_path: &'static str = "./waldo.webp";
    let img = ImageReader::open(image_path)?.decode()?;
    println!(
        "Read image at {} with size: {} x {}",
        image_path,
        img.width(),
        img.height()
    );

    // Copy the image into a vector of pixels, and create a Merkle tree committing to it.
    let img_dimensions = img.dimensions();
    let img_pixels: Vec<[u8; 3]> = img
        .into_rgb8()
        .pixels()
        .map(|pixel: &Rgb<u8>| pixel.0)
        .collect();
    let img_merkle_tree = MerkleTree::<[u8; 3]>::new(img_pixels);

    // Make the prover, loading the image crop method binary and method ID, and registerig a
    // send_recv callback to communicate vector oracle data from the Merkle tree.
    let method_code = std::fs::read(IMAGE_CROP_PATH)?;
    let prover_opts = ProverOpts::default().with_sendrecv_callback(
        VECTOR_ORACLE_CHANNEL,
        img_merkle_tree.vector_oracle_callback(),
    );
    let mut prover = Prover::new_with_opts(&method_code, IMAGE_CROP_ID, prover_opts)?;

    println!(
        "Created Merkle tree with root {:?} and {} leaves",
        img_merkle_tree.root(),
        img_merkle_tree.leafs(),
    );

    // Send the merkle proof to the guest.
    let input = PrivateInput {
        root: img_merkle_tree.root(),
        image_dimensions: img_dimensions,
        crop_locaction: (1160, 300),
        crop_dimensions: (1, 1),
    };
    prover.add_input(&serde::to_vec(&input)?)?;

    // Run prover & generate receipt
    let receipt = prover.run()?;

    // Verify the receipt.
    receipt.verify(IMAGE_CROP_ID)?;
    let journal_vec = receipt.get_journal_vec()?;

    let journal: Journal = serde::from_slice(journal_vec.as_slice())?;

    // TODO: Write out the image such that the user can look at it.
    println!(
        "Verified that {:?} is a crop of the image with dimensions {:?} and Merkle tree root {:?}",
        journal.subsequence, journal.image_dimensions, journal.root
    );

    Ok(())
}
