#![no_main]

use risc0_zkvm::guest::{env, sha};

risc0_zkvm::guest::entry!(main);

pub fn main() {
    let data: String = env::read();
    let sha = sha::digest(&data.as_bytes());
    env::commit(&sha);
}
