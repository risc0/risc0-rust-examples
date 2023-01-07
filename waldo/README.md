# Where's Waldo

TODO: Fill out this README

IMPORTANT NOTE: Currently this example requires having the `risc0/risc0` repo cloned to
`../../risc0` and checked out to `main`.

## Run this example

First, make sure [rustup](https://rustup.rs) is installed. This project uses a [nightly](https://doc.rust-lang.org/book/appendix-07-nightly-rust.html) version of [Rust](https://doc.rust-lang.org/book/ch01-01-installation.html). The [`rust-toolchain`](rust-toolchain) file will be used by `cargo` to automatically install the correct version.

To build and run this example, try using the following commands.

```
# Prove that you know where Waldo is in waldo.webp
cargo run --release --bin prove -- -i waldo.webp -x 1150 -y 291 -w 58 -h 64

# Verify that the prover actually found Waldo.
cargo run --release --bin verify -- -i waldo.webp -r receipt.bin
```
