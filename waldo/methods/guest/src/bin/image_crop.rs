#![no_main]
//#![no_std]

use risc0_zkvm_guest::{env, sha};

risc0_zkvm_guest::entry!(main);

pub fn main() {
    // Read the image from the host.
    let img_bytes: Vec<u8> = env::read();
    let hash = sha::digest_u8_slice(&img_bytes);
    env::commit(&hash);
}
