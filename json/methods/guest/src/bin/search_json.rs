#![no_main]

use json::parse;
use json_core::Outputs;
use risc0_zkvm_guest::{env, sha};

risc0_zkvm_guest::entry!(main);

pub fn main() {
    let data: String = env::read();
    let data2: String = env::read();
    let sha = sha::digest(&data.as_bytes());
    let sha2 = sha::digest(&data.as_bytes());
    let data = parse(&data).unwrap();
    let data2 = parse(&data2).unwrap();
    if data["critical_data"].as_u32().unwrap() != data2["critical_data"].as_u32().unwrap() {
        panic!()
    }
    let out = Outputs {
        hash: *sha,
        hash2: *sha2,
    };
    env::commit(&out);
}
