#![no_main]

use json::parse;
use json_core::Outputs;
use risc0_zkvm::guest::{env, sha};

risc0_zkvm::guest::entry!(main);

pub fn main() {
    let data: String = env::read();
    let sha = sha::digest(&data.as_bytes());
    let data = parse(&data).unwrap();
    let proven_val = data["critical_data"].as_u32().unwrap();
    let out = Outputs {
        data: proven_val,
        hash: *sha,
    };
    env::commit(&out);
}
