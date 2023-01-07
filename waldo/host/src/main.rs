use std::error::Error;
use image::io::Reader as ImageReader;
use image::{RgbImage, GenericImageView, ImageFormat};
use risc0_zkvm::prove::{Prover, ProverOpts};
use risc0_zkvm::serde;
use waldo_core::merkle::VECTOR_ORACLE_CHANNEL;
use waldo_core::image::{IMAGE_CHUNK_SIZE, ImageMerkleTree};
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

    let img_merkle_tree = ImageMerkleTree::<{IMAGE_CHUNK_SIZE}>::new(&img);

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
    let crop_dimensions = (58, 64);
    let crop_location = (1150, 291);
    let input = PrivateInput {
        root: img_merkle_tree.root(),
        image_dimensions: img.dimensions(),
        crop_location,
        crop_dimensions,
    };
    prover.add_input_u32_slice(&serde::to_vec(&input)?);

    // Run prover & generate receipt
    let receipt = prover.run()?;

    // Verify the receipt.
    receipt.verify(IMAGE_CROP_ID)?;
    let journal: Journal = serde::from_slice(&receipt.journal)?;

    println!(
        "Verified an with dimensions {:?} is a crop of the image with dimensions {:?} and Merkle tree root {:?}",
        journal.subimage_dimensions, journal.image_dimensions, &journal.root
    );

    let subimage = RgbImage::from_raw(journal.subimage_dimensions.0, journal.subimage_dimensions.1, journal.subimage).ok_or("failed to load the returned subimage bytes into an image")?;
    subimage.save_with_format("./waldo_cropped.png", ImageFormat::Png)?;

    Ok(())
}
