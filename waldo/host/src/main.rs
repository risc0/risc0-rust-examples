use std::error::Error;

use image::io::Reader as ImageReader;
use image::ImageOutputFormat;
use rand::RngCore;
use risc0_zkvm::host::{Prover, ProverOpts};
use risc0_zkvm::serde;
use waldo_core::merkle::MerkleTree;
use waldo_core::{Journal, PrivateInput, VECTOR_ORACLE_CHANNEL};
use waldo_methods::{IMAGE_CROP_ID, IMAGE_CROP_PATH};

fn main() -> Result<(), Box<dyn Error>> {
    /*

    // Pick two numbers
    let image_path: &'static str = "./waldo.webp";
    let img = ImageReader::open(image_path)?.decode()?;
    println!("Read image at {} with size: {} x {}", image_path, img.width(), img.height());

    let mut cursor = Cursor::new(Vec::new());
    let format = ImageOutputFormat::Bmp;
    let img_rgb8 = img.as_rgb8().ok_or("cannot encode image as RGB8")?;
    img_rgb8.write_to(&mut cursor, format.clone())?;
    let img_bytes = cursor.into_inner();
    println!("Wrote image to a buffer with format {:?} and {} bytes", &format, &img_bytes.len());
    */

    // Fill a buffer with random bytes.
    let mut img_bytes = vec![0u8; 1 << 15];
    rand::thread_rng().fill_bytes(&mut img_bytes);

    // Create a Merkle tree over the image bytes.
    // TODO: Chunk the bytes into reasonable sizes.
    let img_bytes_merkle_tree = MerkleTree::<u8>::from_elements(img_bytes.iter().copied());

    // Make the prover, loading the image crop method binary and method ID.
    let method_code = std::fs::read(IMAGE_CROP_PATH)?;
    let prover_opts = ProverOpts::default().with_sendrecv_callback(
        VECTOR_ORACLE_CHANNEL,
        |channel_id, data| -> Vec<u8> {
            assert_eq!(channel_id, VECTOR_ORACLE_CHANNEL);
            // NOTE: This would be nicer with we could avoid bytemuck.
            let index: usize = serde::from_slice::<u32>(bytemuck::cast_slice(data))
                .unwrap()
                .try_into()
                .unwrap();
            let value = img_bytes[index];
            let proof = img_bytes_merkle_tree.prove(index);
            bytemuck::cast_slice(&serde::to_vec(&(value, proof)).unwrap()).to_vec()
        },
    );
    let mut prover = Prover::new_with_opts(&method_code, IMAGE_CROP_ID, prover_opts)?;

    println!(
        "Created Merkle tree with root {:?} and {} leaves",
        img_bytes_merkle_tree.root(),
        img_bytes_merkle_tree.leafs(),
    );

    // Send the merkle proof to the guest.
    let input = PrivateInput {
        root: img_bytes_merkle_tree.root(),
        range: 157..167,
    };
    prover.add_input(&serde::to_vec(&input)?)?;

    // Run prover & generate receipt
    let receipt = prover.run()?;

    // Verify the receipt.
    receipt.verify(IMAGE_CROP_ID)?;
    let journal_vec = receipt.get_journal_vec()?;

    let journal: Journal = serde::from_slice(journal_vec.as_slice())?;

    println!(
        "Verified that {:?} is a subsequence of a Merkle tree with root: {:?}",
        journal.subsequence, journal.root,
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
