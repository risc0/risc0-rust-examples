#![no_main]

use risc0_zkvm_guest::{env, sha};

risc0_zkvm_guest::entry!(main);

pub fn main() {
    let data: String = env::read();
    let sha = sha::digest_u8_slice(&data.as_bytes());
    env::commit(&sha);
}
