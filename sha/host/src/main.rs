// TODO: Update the name of the method loaded by the prover. E.g., if the method is `multiply`, replace `METHOD_NAME_ID` with `MULTIPLY_ID` and replace `METHOD_NAME_PATH` with `MULTIPLY_PATH`
use methods::{HASH_ID, HASH_PATH};
use risc0_zkvm::host::Prover;
use risc0_zkvm::serde::{from_slice, to_vec};
use risc0_zkp::core::sha::Digest;

fn main() {
    // Make the prover.
    let method_code = std::fs::read(HASH_PATH)
        .expect("Method code should be present at the specified path");
    let mut prover = Prover::new(&method_code, HASH_ID)
        .expect("Prover should be constructed from matching code and method ID");

    prover.add_input(&to_vec("abc").expect("abc should serialize")).expect("Prover should accept input");

    // Run prover & generate receipt
    let receipt = prover.run()
        .expect("Code should be provable");
    receipt.verify(HASH_ID)
        .expect("Proven code should verify");

    let vec = receipt.get_journal_vec().expect("Journal should be accessible");
    let digest = from_slice::<Digest>(vec.as_slice()).expect("Journal should contain SHA Digest");

    println!("The hash is {}", digest);
}
