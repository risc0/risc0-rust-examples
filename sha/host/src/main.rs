// TODO: Update the name of the method loaded by the prover. E.g., if the method is `multiply`, replace `METHOD_NAME_ID` with `MULTIPLY_ID` and replace `METHOD_NAME_PATH` with `MULTIPLY_PATH`
use methods::{HASH_ID, HASH_PATH};
use risc0_zkvm::host::Prover;
use risc0_zkvm::serde::{from_slice, to_vec};
use risc0_zkp::core::sha::Digest;

fn main() {
    // Make the prover.
    let method_code = std::fs::read(HASH_PATH)
        .expect("Method code should be present at the specified path; did you use the correct *_PATH constant?");
    let mut prover = Prover::new(&method_code, HASH_ID)
        .expect("Prover should be constructed from valid method source code and corresponding method ID");

    prover.add_input(&to_vec("abc").unwrap()).unwrap();

    // Run prover & generate receipt
    let receipt = prover.run()
        .expect("Valid code should be provable if it doesn't overflow the cycle limit. See `embed_methods_with_options` for information on adjusting maximum cycle count.");
    receipt.verify(HASH_ID)
        .expect("Code you have proven should successfully verify; did you specify the correct method ID?");

    let vec = receipt.get_journal_vec().unwrap();
    let digest = from_slice::<Digest>(vec.as_slice()).unwrap();

    println!("The hash is {}", digest);
}
