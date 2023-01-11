use std::error::Error;
use std::fs;
use std::path::PathBuf;

use clap::Parser;
use image::io::Reader as ImageReader;
use image::GenericImageView;
use risc0_zkvm::prove::{Prover, ProverOpts};
use risc0_zkvm::serde;
use waldo_core::image::{ImageMask, ImageMerkleTree, IMAGE_CHUNK_SIZE};
use waldo_core::merkle::VECTOR_ORACLE_CHANNEL;
use waldo_core::PrivateInput;
use waldo_methods::{IMAGE_CROP_ID, IMAGE_CROP_PATH};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Input file path to the full Where's Waldo image.
    #[clap(short = 'i', long, value_parser, value_hint = clap::ValueHint::FilePath)]
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

    /// Optional input file path to an image mask to apply to Waldo.
    /// Grayscale pixel values will be subtracted from the cropped image of Waldo such that a black
    /// pixel in the mask will result in the cooresponding image pixel being blacked out.
    /// Must be the same dimensions, in pixels, as the cut out x and y.
    #[clap(short = 'm', long, value_parser, value_hint = clap::ValueHint::FilePath)]
    mask: Option<PathBuf>,

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

    let crop_location = (args.waldo_x, args.waldo_y);
    let crop_dimensions = (args.waldo_width, args.waldo_height);

    // Read the image mask from disk, if provided.
    let mask = args.mask.map_or(Ok::<_, Box<dyn Error>>(None), |path| {
        // Read the image mask from disk. Reads any format and color image.
        let mask: ImageMask = ImageReader::open(&path)?.decode()?.into();
        if mask.dimensions() != crop_dimensions {
            return Err(format!(
                "Mask dimensions do not match specified height and width for Waldo: {:?} != {:?}",
                mask.dimensions(),
                crop_dimensions
            )
            .into());
        }
        println!("Read image mask at {}", &path.display(),);

        Ok(Some(mask.into_raw()))
    })?;

    // Construct a Merkle tree from the full Where's Waldo image.
    let img_merkle_tree = ImageMerkleTree::<{ IMAGE_CHUNK_SIZE }>::new(&img);
    println!(
        "Created Merkle tree from image with root {:?}",
        img_merkle_tree.root(),
    );

    // Make the prover, loading the image crop method binary and method ID, and registering a
    // send_recv callback to communicate vector oracle data from the Merkle tree.
    let method_code = std::fs::read(IMAGE_CROP_PATH)?;
    let prover_opts = ProverOpts::default().with_sendrecv_callback(
        VECTOR_ORACLE_CHANNEL,
        img_merkle_tree.vector_oracle_callback(),
    );
    let mut prover = Prover::new_with_opts(&method_code, IMAGE_CROP_ID, prover_opts)?;

    // Give the private input to the guest, including Waldo's location.
    let input = PrivateInput {
        root: img_merkle_tree.root(),
        image_dimensions: img.dimensions(),
        mask,
        crop_location,
        crop_dimensions,
    };
    prover.add_input_u32_slice(&serde::to_vec(&input)?);

    // Run prover and generate receipt
    println!(
        "Running the prover to cut out Waldo at {:?} with dimensions {:?}",
        input.crop_location, input.crop_dimensions,
    );
    let receipt = prover.run()?;

    // Save the receipt to disk so it can be sent to the verifier.
    fs::write(&args.receipt, bincode::serialize(&receipt)?)?;

    println!("Success! Saved the receipt to {}", &args.receipt.display());

    Ok(())
}
