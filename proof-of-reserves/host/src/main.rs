// TODO: Update the name of the method loaded by the prover. E.g., if the method is `multiply`, replace `METHOD_NAME_ID` with `MULTIPLY_ID` and replace `METHOD_NAME_PATH` with `MULTIPLY_PATH`
use bitcoin::util::{address::Address, key::PublicKey};
use bitcoin::blockdata::{script::Script, transaction::TxOut};
use methods::{METHOD_NAME_ID, METHOD_NAME_PATH};
use risc0_zkvm::Prover;
// use risc0_zkvm::serde::{from_slice, to_vec};
use secp256k1::{Secp256k1, SecretKey};

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

// This `derive` requires the `serde` dependency.
#[derive(Debug, serde::Deserialize)]
struct MerklePathResponse {
    result: String,
}

fn get_tx_out_merkle_proof(tx_hash: String, rpc_url: String) -> String {
    let client = reqwest::blocking::Client::new();
    let proof_json_test: String = client
        .post(rpc_url)
        .body(format!(
            "{{\"method\": \"gettxoutproof\", \"params\": [[\"{}\"]]}}",
            tx_hash
        ))
        .send()
        .unwrap()
        .text()
        .unwrap();

    let proof: MerklePathResponse = serde_json::from_str(&proof_json_test).unwrap();
    proof.result
}

fn private_key_to_address(private_key: String) -> Address {
    let bitcoin_public_key = public_key_from_private_key(private_key);

    bitcoin::util::address::Address::p2pkh(
        &bitcoin_public_key,
        bitcoin::network::constants::Network::Bitcoin,
    )
}

fn public_key_from_private_key(private_key: String) -> PublicKey {
    let private_key_bytes = hex::decode(private_key).unwrap();

    let secp = Secp256k1::new();
    let secret_key =
        SecretKey::from_slice(&private_key_bytes).expect("32 bytes, within curve order");
    PublicKey::new(secp256k1::PublicKey::from_secret_key(&secp, &secret_key))
}

fn verify_ownership(txout: TxOut, private_key: String) -> Result<(), ()> {
    let public_key = public_key_from_private_key(private_key);

    if txout.script_pubkey != Script::new_p2pkh(&public_key.pubkey_hash()) {
        return Err(());
    }
    Ok(())
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
    fn verify_txout_ownership_works() {
        let private_key =
            "18E14A7B6A307F426A94F8114701E7C8E774E7F9A47E2C2035DB29A206321725".to_string();
        let public_key = public_key_from_private_key(private_key.clone());
        let pubkey_hash = public_key.pubkey_hash();
        let txout = TxOut {
            value: 47,
            script_pubkey: Script::new_p2pkh(&pubkey_hash)
        };
        // Ownership successfully verifies
        verify_ownership(txout.clone(), private_key).unwrap();
        // Ownership fails verification
        assert_eq!(verify_ownership(txout, "00004A7B6A307F426A94F8114701E7C8E774E7F9A47E2C2035DB29A206321725".to_string()), Err(()));
    }
}
