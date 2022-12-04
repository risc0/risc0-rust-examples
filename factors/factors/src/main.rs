use methods::{MULTIPLY_ID, MULTIPLY_PATH};
use risc0_zkvm::host::Prover;
use risc0_zkvm::serde::{from_slice, to_vec};

fn main() {
    // Pick two numbers
    let a: u64 = 17;
    let b: u64 = 23;

    // Multiply them inside the ZKP
    let multiply_src = std::fs::read(MULTIPLY_PATH).unwrap();
    let mut prover = Prover::new(&multiply_src, MULTIPLY_ID).unwrap();
    prover.add_input(&to_vec(&a).unwrap()).unwrap();
    prover.add_input(&to_vec(&b).unwrap()).unwrap();
    let receipt = prover.run().unwrap();

    receipt.verify(MULTIPLY_ID).unwrap();

    // Extract journal of receipt (i.e. output c, where c = a * b)
    let c: u64 = from_slice(&receipt.get_journal_vec().unwrap()).unwrap();
    println!("I know the factors of {}, and I can prove it!", c);
}
