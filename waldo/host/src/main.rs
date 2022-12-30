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

    println!(
        "Verified that {:?} is a subsequence of a Merkle tree with root: {:?}, {:?}",
        journal.subsequence, journal.root, journal.image_dimensions
    );

    Ok(())
    /*
    // Multiply them inside the ZKP
    // First, we make the prover, loading the 'multiply' method
    let multiply_src = std::fs::read(MULTIPLY_PATH)
        .expect("Method code should be present at the specified path; did you use the correct *_PATH constant?");
    let mut prover = Prover::new(&multiply_src, MULTIPLY_ID).expect(
        "Prover should be constructed from valid method source code and corresponding method ID",
    );

    // Next we send a & b to the guest
    prover.add_input(to_vec(&a).unwrap().as_slice()).unwrap();
    prover.add_input(to_vec(&b).unwrap().as_slice()).unwrap();
    // Run prover & generate receipt
    let receipt = prover.run()
        .expect("Valid code should be provable if it doesn't overflow the cycle limit. See `embed_methods_with_options` for information on adjusting maximum cycle count.");

    // Extract journal of receipt (i.e. output c, where c = a * b)
    let c: u64 = from_slice(
        &receipt
            .get_journal_vec()
            .expect("Journal should be available for valid receipts"),
    )
    .expect("Journal output should deserialize into the same types (& order) that it was written");

    // Print an assertion
    println!("I know the factors of {}, and I can prove it!", c);

    // Here is where one would send 'receipt' over the network...

    // Verify receipt, panic if it's wrong
    receipt.verify(MULTIPLY_ID).expect(
        "Code you have proven should successfully verify; did you specify the correct method ID?",
    );
    */
}
