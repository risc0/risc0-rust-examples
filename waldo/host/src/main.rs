use std::error::Error;
use std::ops::Deref;

use image::io::Reader as ImageReader;
use image::{DynamicImage, GenericImageView};
use risc0_zkvm::prove::{Prover, ProverOpts};
use risc0_zkvm::serde;
use waldo_core::{IMAGE_CHUNK_SIZE, merkle::{MerkleTree, VECTOR_ORACLE_CHANNEL}};
use waldo_core::{Journal, PrivateInput};
use waldo_methods::{IMAGE_CROP_ID, IMAGE_CROP_PATH};

/// ImageMerkleTree is a merklization of an image, constructed with the leaf elements being NxN
/// square chunks.
///
/// Chunks on the right and bottom boundaries will be incomplete if the width or
/// height cannot be divided by N.
pub struct ImageMerkleTree<const N: u32>(MerkleTree<Vec<u8>>);

impl<const N: u32> ImageMerkleTree<N> {
    pub fn new(image: &DynamicImage) -> Self {
        // Iterate over the NxN chunks of an image in right to left, top to bottom, order.
        // Convert the image into RGB8 as it is chunked. Access to the image will be to the
        // underlying subpixels (i.e. bytes for RGB8).
        let chunks: Vec<Vec<u8>> = {
            (0..image.height())
                .step_by(usize::try_from(N).unwrap())
                .map(|y| {
                    (0..image.width())
                        .step_by(usize::try_from(N).unwrap())
                        .map(move |x| image.crop_imm(x, y, N, N).into_rgb8().into_raw())
                })
                .flatten()
                .collect()
        };

        Self(MerkleTree::new(chunks))
    }
}

impl<const N: u32> Deref for ImageMerkleTree<N> {
    type Target = MerkleTree<Vec<u8>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

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
    let input = PrivateInput {
        root: img_merkle_tree.root(),
        image_dimensions: img.dimensions(),
        crop_location: (1150, 291),
        crop_dimensions: (58, 64),
    };
    prover.add_input_u32_slice(&serde::to_vec(&input)?);

    // Run prover & generate receipt
    let receipt = prover.run()?;

    // Verify the receipt.
    receipt.verify(IMAGE_CROP_ID)?;
    let journal: Journal = serde::from_slice(&receipt.journal)?;

    // TODO: Write out the image such that the user can look at it.
    println!(
        "Verified that {:?} is a crop of the image with dimensions {:?} and Merkle tree root {:?}",
        journal.subimage, journal.image_dimensions, journal.root
    );

    Ok(())
}
