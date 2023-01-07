use std::error::Error;
use std::fs;
use std::path::PathBuf;

use clap::Parser;
use image::io::Reader as ImageReader;
use image::GenericImageView;
use risc0_zkvm::prove::{Prover, ProverOpts};
use risc0_zkvm::serde;
use waldo_core::image::{ImageMerkleTree, IMAGE_CHUNK_SIZE};
use waldo_core::merkle::VECTOR_ORACLE_CHANNEL;
use waldo_core::PrivateInput;
use waldo_methods::{IMAGE_CROP_ID, IMAGE_CROP_PATH};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Input file path to the full Where's Waldo image.
    #[clap(short, long, value_parser, value_hint = clap::ValueHint::FilePath)]
    image: PathBuf,

    /// X coordinate, in pixels from the top-left corner, of Waldo.
    #[clap(short = 'x', long, value_parser)]
    waldo_x: u32,

    /// Y coordinate, in pixels from the top-left corner, of Waldo.
    #[clap(short = 'y', long, value_parser)]
    waldo_y: u32,

    /// Width, in pixels, of the cutout for Waldo.
    #[clap(short = 'w', long, value_parser)]
    waldo_width: u32,

    /// Height, in pixels, of the cutout for Waldo.
    #[clap(short = 'h', long, value_parser)]
    waldo_height: u32,

    /// Output file path to save the receipt. Note that the receipt contains the cutout of waldo.
    #[clap(short = 'r', long, value_parser, default_value = "./receipt.bin", value_hint = clap::ValueHint::FilePath)]
    receipt: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    // Read the image from disk.
    let img = ImageReader::open(&args.image)?.decode()?;
    println!(
        "Read image at {} with size: {} x {}",
        &args.image.display(),
        img.width(),
        img.height()
    );

    // Construct a Merkle tree from the Where's Waldo image.
    let img_merkle_tree = ImageMerkleTree::<{ IMAGE_CHUNK_SIZE }>::new(&img);

    // Make the prover, loading the image crop method binary and method ID, and registering a
    // send_recv callback to communicate vector oracle data from the Merkle tree.
    let method_code = std::fs::read(IMAGE_CROP_PATH)?;
    let prover_opts = ProverOpts::default().with_sendrecv_callback(
        VECTOR_ORACLE_CHANNEL,
        img_merkle_tree.vector_oracle_callback(),
    );
    let mut prover = Prover::new_with_opts(&method_code, IMAGE_CROP_ID, prover_opts)?;

    println!(
        "Created Merkle tree from image with root {:?}",
        img_merkle_tree.root(),
    );

    // Send the merkle proof to the guest.
    let crop_location = (args.waldo_x, args.waldo_y);
    let crop_dimensions = (args.waldo_width, args.waldo_height);
    let input = PrivateInput {
        root: img_merkle_tree.root(),
        image_dimensions: img.dimensions(),
        crop_location,
        crop_dimensions,
    };
    prover.add_input_u32_slice(&serde::to_vec(&input)?);

    println!(
        "Running the prover to cut out waldo at {:?} with dimensions {:?}",
        input.crop_location, input.crop_dimensions,
    );

    // Run prover and generate receipt
    let receipt = prover.run()?;

    // Save the receipt to disk so it can be sent to the verifier.
    fs::write(&args.receipt, bincode::serialize(&receipt)?)?;

    println!("Success! Saved the receipt to {}", &args.receipt.display());

    Ok(())
}
