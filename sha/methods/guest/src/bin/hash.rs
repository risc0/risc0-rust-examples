#![no_main]

use risc0_zkvm::guest::env;
use risc0_zkvm::sha::{sha, Sha};

risc0_zkvm::guest::entry!(main);

pub fn main() {
    let data: String = env::read();
    let digest = sha().hash_bytes(&data.as_bytes());
    env::commit(&*digest);
}
