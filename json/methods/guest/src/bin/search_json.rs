#![no_main]

use json::parse;
use risc0_zkvm_guest::{env, sha};

risc0_zkvm_guest::entry!(main);

pub fn main() {
    let data: String = env::read();
    let sha = sha::digest(&data.as_bytes());
    let data = parse(&data).unwrap();
    let proven_val = data["critical_data"].as_u32().unwrap();
    env::commit(&proven_val);
    env::commit(&sha);
}
