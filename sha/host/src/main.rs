use clap::{Arg, Command};
use methods::{HASH_ID, HASH_PATH};
use risc0_zkp::core::sha::Digest;
use risc0_zkvm::prove::Prover;
use risc0_zkvm::receipt::Receipt;
use risc0_zkvm::serde::{from_slice, to_vec};

fn provably_hash(input: &str) -> Receipt {
    // Make the prover.
    let method_code =
        std::fs::read(HASH_PATH).expect("Method code should be present at the specified path");
    let mut prover = Prover::new(&method_code, HASH_ID)
        .expect("Prover should be constructed from matching code and method ID");

    prover.add_input_u32_slice(&to_vec(input).expect("input string should serialize"));

    // Run prover & generate receipt
    prover.run().expect("Code should be provable")
}

fn main() {
    // Parse command line
    let matches = Command::new("hash")
        .arg(Arg::new("message").default_value(""))
        .get_matches();
    let message = matches.get_one::<String>("message").unwrap();

    // Prove hash and verify it
    let receipt = provably_hash(message);
    receipt.verify(HASH_ID).expect("Proven code should verify");

    let digest = from_slice::<Digest>(receipt.journal.as_slice())
        .expect("Journal should contain SHA Digest");

    println!("I provably know data whose SHA-256 hash is {}", digest);
}

#[cfg(test)]
mod tests {
    use methods::HASH_ID;
    use risc0_zkp::core::sha::Digest;
    use risc0_zkvm::serde::from_slice;

    use crate::provably_hash;
    use sha2::Digest as Sha2Digest;

    #[test]
    fn main() {
        let receipt = provably_hash("abc");
        receipt.verify(HASH_ID).expect("Proven code should verify");

        let digest = from_slice::<Digest>(receipt.journal.as_slice())
            .expect("Journal should contain SHA Digest");
        assert_eq!(
            digest.as_bytes(),
            sha2::Sha256::digest("abc").as_slice(),
            "We expect to match the reference SHA-256 hash of the standard test value 'abc'"
        );
    }
}
