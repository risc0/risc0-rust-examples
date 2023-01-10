use std::error::Error;
use std::fs;
use std::path::PathBuf;

use clap::Parser;
use image::io::Reader as ImageReader;
use image::{GenericImageView, RgbImage};
use risc0_zkvm::receipt::Receipt;
use risc0_zkvm::serde;
use waldo_core::image::{ImageMerkleTree, IMAGE_CHUNK_SIZE};
use waldo_core::Journal;
use waldo_methods::IMAGE_CROP_ID;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Input file path to the full Where's Waldo image.
    /// Used to verify that the Waldo in the recipt actually came from this image.
    #[clap(short, long, value_parser, value_hint = clap::ValueHint::FilePath)]
    image: PathBuf,

    /// Input file path to fetch the recipt. Note that the receipt contains the cutout of waldo.
    #[clap(short = 'r', long, value_parser, default_value = "./receipt.bin", value_hint = clap::ValueHint::FilePath)]
    receipt: PathBuf,

    /// Output file path to save the cutout image of Waldo extracted from the receipt.
    /// SAFETY: Make sure to visually inspect the Waldo cutout and verify it really is Waldo and
    /// not some barber pole!
    #[clap(short = 'o', long, value_parser, default_value = "./waldo_cutout.png", value_hint = clap::ValueHint::FilePath)]
    waldo: PathBuf,
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

    println!(
        "Created Merkle tree from image with root {:?}",
        img_merkle_tree.root(),
    );

    // Load and verify the receipt file to get the journal.
    let receipt: Receipt = bincode::deserialize(&fs::read(&args.receipt)?)?;
    receipt.verify(IMAGE_CROP_ID)?;
    let journal: Journal = serde::from_slice(&receipt.journal)?;

    // Check consistency of the journal against the input Where's Waldo image.
    if &journal.root != &img_merkle_tree.root() {
        return Err(format!(
            "Image root in journal does not match the expected image: {:?} != {:?}",
            &journal.root,
            &img_merkle_tree.root(),
        )
        .into());
    }

    if journal.image_dimensions != img.dimensions() {
        return Err(format!(
            "Image dimensions in the journal do not match the expected image: {:?} != {:?}",
            journal.image_dimensions,
            img.dimensions(),
        )
        .into());
    }

    println!(
        "Verified receipt with {}x{} subimage",
        journal.subimage_dimensions.0, journal.subimage_dimensions.1
    );

    let subimage = RgbImage::from_raw(
        journal.subimage_dimensions.0,
        journal.subimage_dimensions.1,
        journal.subimage,
    )
    .ok_or("failed to load the returned subimage bytes into an image")?;

    // Save the image to disk for the verifier to inspect.
    subimage.save(&args.waldo)?;
    println!("Saved Waldo cutout to {}", &args.waldo.display());

    // Display the image in the terminal for them to see whether it's Waldo.
    let viuer_config = viuer::Config {
        absolute_offset: false,
        ..Default::default()
    };
    viuer::print_from_file(&args.waldo, &viuer_config)?;
    println!("Prover knows where this cutout is in the given image.");
    println!("Do you recognize this Waldo?");

    Ok(())
}
