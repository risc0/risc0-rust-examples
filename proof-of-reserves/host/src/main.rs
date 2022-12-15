// TODO: Update the name of the method loaded by the prover. E.g., if the method is `multiply`, replace `METHOD_NAME_ID` with `MULTIPLY_ID` and replace `METHOD_NAME_PATH` with `MULTIPLY_PATH`
use bitcoin::hashes::hex::FromHex;
use bitcoin::util::address::Address;
use bitcoin::{MerkleBlock, Txid};
use methods::{METHOD_NAME_ID, METHOD_NAME_PATH};
use reqwest::blocking::Client;
use risc0_zkvm::Prover;
// use risc0_zkvm::serde::{from_slice, to_vec};
use secp256k1::{PublicKey, Secp256k1, SecretKey};

fn main() {
    // Make the prover.
    let method_code = std::fs::read(METHOD_NAME_PATH)
        .expect("Method code should be present at the specified path; did you use the correct *_PATH constant?");
    let mut prover = Prover::new(&method_code, METHOD_NAME_ID).expect(
        "Prover should be constructed from valid method source code and corresponding method ID",
    );

    // TODO: Implement communication with the guest here

    // Run prover & generate receipt
    let receipt = prover.run()
        .expect("Code should be provable unless it 1) had an error or 2) overflowed the cycle limit. See `embed_methods_with_options` for information on adjusting maximum cycle count.");

    // Optional: Verify receipt to confirm that recipients will also be able to verify your receipt
    receipt.verify(METHOD_NAME_ID).expect(
        "Code you have proven should successfully verify; did you specify the correct method ID?",
    );

    // TODO: Implement code for transmitting or serializing the receipt for other parties to verify here
}

fn private_key_to_address(private_key: String) -> Address {
    let private_key_bytes = hex::decode(private_key).unwrap();

    let secp = Secp256k1::new();
    let secret_key =
        SecretKey::from_slice(&private_key_bytes).expect("32 bytes, within curve order");
    let public_key = PublicKey::from_secret_key(&secp, &secret_key);

    let bitcoin_public_key = bitcoin::util::key::PublicKey::new(public_key);
    bitcoin::util::address::Address::p2pkh(
        &bitcoin_public_key,
        bitcoin::network::constants::Network::Bitcoin,
    )
}

fn verify_merkle_proof(expected_tx_hash: String, expected_tx_index: u32, merkle_proof: String) {
    let mb: MerkleBlock =
        bitcoin::consensus::deserialize(&Vec::from_hex(&merkle_proof).unwrap()).unwrap();
    let mut matches: Vec<Txid> = vec![];
    let mut index: Vec<u32> = vec![];
    assert!(mb.extract_matches(&mut matches, &mut index).is_ok());
    assert_eq!(1, matches.len());
    assert_eq!(Txid::from_hex(&expected_tx_hash).unwrap(), matches[0]);
    assert_eq!(1, index.len());
    assert_eq!(expected_tx_index, index[0]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn private_key_to_public_key_works() {
        let private_key =
            "18E14A7B6A307F426A94F8114701E7C8E774E7F9A47E2C2035DB29A206321725".to_string();
        let address = private_key_to_address(private_key);
        assert_eq!(address.to_string(), "1PMycacnJaSqwwJqjawXBErnLsZ7RkXUAs");
    }

    #[test]
    fn verify_merkle_proof_works() {
        let proof =
            "01000000ba8b9cda965dd8e536670f9ddec10e53aab14b20bacad27b9137190000000000190760b278fe7b8565fda3b968b918d5fd997f993b23674c0af3b6fde300b38f33a5914ce6ed5b1b01e32f570200000002252bf9d75c4f481ebb6278d708257d1f12beb6dd30301d26c623f789b2ba6fc0e2d32adb5f8ca820731dff234a84e78ec30bce4ec69dbd562d0b2b8266bf4e5a0105".to_string();
        let expected_tx_hash =
            "5a4ebf66822b0b2d56bd9dc64ece0bc38ee7844a23ff1d7320a88c5fdb2ad3e2".to_string();
        let expected_tx_index = 1;

        verify_merkle_proof(expected_tx_hash, expected_tx_index, proof)
    }
}
