# Where's Waldo

[Where's Waldo] is a [favorite analogy] for zero-knowledge proofs. In particular, there is this
visual that if you take a Where's Waldo image and cover it up with a big piece of cardboard with a
small cutout that just shows Waldo, you can prove you _know_ where he is while keeping that location
secret.

[Where's Waldo]: https://en.wikipedia.org/wiki/Where%27s_Wally%3F  
[favorite analogy]: https://medium.com/swlh/a-zero-knowledge-proof-for-wheres-wally-930c21e55399

But these days, why not implement a real zero-knowledge proof to show you know where Waldo is?

This example implements a RISC0 zero-knowledge proof which allows a prover to convince a verifier
they know where Waldo is in a public Where's Waldo puzzle, without revealing Waldo's coordinates.

## Approach

The approach for this example is similar to the analogy. It take the full image and "cuts out" just
Waldo. This cutting out operation takes place in the zkvm guest such that a commitment to the source
image and the cut out image of Waldo can be revealed, without giving the verifier the coordinates.
Key to this is ensuring that the cutout came from the expected source image.

### Merkleization

In the simplest approach, the guest program would simply hash the whole Where's Waldo image in
memory, then perform the crop and mask operations to cut out Waldo on the image that was just
hashed, committing to the image hash and the output. Unfortunately, hashing the whole image, which
we expect to be rather large, if cost prohibitive in the guest.

Because we only need access to a relatively small portion of the image to produce the cutout, a
viable approach is to split the image into a vector of small image chunks and use a Merkle tree to
commit to this vector. The zkvm guest can then ask the host for image chunks, and along with the
chunk the host can provide a Merkle path that proves the chunk is part of the committed image.

In the `waldo_core::merkle` module is implemented a wrapper on the `merkle_light` crate with support
for using the SHA256 guest circuit, and providing a `VectorOracle` abstraction. In the
`waldo_core::image` module is implemented a specific MerkleTree type for images, and an
`ImageOracle` type which can be used in the guest for image operations.

### Image Manipulation

In order to manipulate the image and cut out waldo, in particular cropping and applying a mask, this
example utilizes the popular `image` crate. This is enabled by implementing
`image::GenericImageView` on `ImageOracle`. With that trait, many of the image operations provided
in the `image` crate, and by others, can be used on `ImageOracle` inside the guest. A simmilar
approach could be used to produce a provable blur, image down-scaling, and more.

## Run this example

First, make sure [rustup](https://rustup.rs) is installed.
This project uses a [nightly](https://doc.rust-lang.org/book/appendix-07-nightly-rust.html) version of [Rust](https://doc.rust-lang.org/book/ch01-01-installation.html).
The [`rust-toolchain`](rust-toolchain) file will be used by `cargo` to automatically install the correct version.

To build and run this example, try using the following commands.

```bash
# Prove that you know where Waldo is in waldo.webp
cargo run --release --bin prove -- -i waldo.webp -x 1150 -y 291 -w 58 -h 70 -m waldo_mask.png

# Verify that the prover actually found Waldo.
cargo run --release --bin verify -- -i waldo.webp -r receipt.bin
```
