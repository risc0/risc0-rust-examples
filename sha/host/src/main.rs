use clap::{Arg, Command};
use methods::{HASH_ID, HASH_PATH};
use risc0_zkp::core::sha::Digest;
use risc0_zkvm::host::{Prover, Receipt};
use risc0_zkvm::serde::{from_slice, to_vec};

fn provably_hash(input: &str) -> Receipt {
    // Make the prover.
    let method_code =
        std::fs::read(HASH_PATH).expect("Method code should be present at the specified path");
    let mut prover = Prover::new(&method_code, HASH_ID)
        .expect("Prover should be constructed from matching code and method ID");

    prover
        .add_input(&to_vec(input).expect("input string should serialize"))
        .expect("Prover should accept input");

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

    let vec = receipt
        .get_journal_vec()
        .expect("Journal should be accessible");
    let digest = from_slice::<Digest>(vec.as_slice()).expect("Journal should contain SHA Digest");

    println!("I provably know data whose SHA-256 hash is {}", digest);
}

#[cfg(test)]
mod tests {
    use methods::{HASH_ID, HASH_PATH};
    use risc0_zkp::core::sha::Digest;
    use risc0_zkvm::host::Prover;
    use risc0_zkvm::serde::{from_slice, to_vec};

    use crate::provably_hash;

    #[test]
    fn main() {
        let receipt = provably_hash("abc");
        receipt.verify(HASH_ID).expect("Proven code should verify");

        let vec = receipt
            .get_journal_vec()
            .expect("Journal should be accessible");
        let digest =
            from_slice::<Digest>(vec.as_slice()).expect("Journal should contain SHA Digest");
        assert_eq!(
            digest,
            Digest::new([
                0xba7816bf, 0x8f01cfea, 0x414140de, 0x5dae2223, 0xb00361a3, 0x96177a9c, 0xb410ff61,
                0xf20015ad
            ]),
            "We expect to match the reference SHA-256 hash of the standard test value 'abc'"
        );
    }
}
