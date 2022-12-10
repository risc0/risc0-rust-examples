#[macro_use]
extern crate static_assertions;

use std::cmp::Ordering;
use std::error::Error;
use std::hash::Hasher;
use std::mem::size_of;

use merkletree::hash::Algorithm;
use merkletree::merkle::{Element, MerkleTree};
use rand::distributions::{Distribution, Standard};
use rand::Rng;
use risc0_zkp::core::sha::{Digest, Sha, DIGEST_WORDS, DIGEST_WORD_SIZE};

// Wrapper on the RISC0 Digest type to allow it to act as a Merkle tree element.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Node(Digest);

const_assert_eq!(size_of::<Node>(), DIGEST_WORDS * DIGEST_WORD_SIZE);

// NOTE: It would be nice is Digest implements Ord and/or Into<[u32; 8]>
impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.get().cmp(other.0.get())
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl AsRef<[u8]> for Node {
    fn as_ref(&self) -> &[u8] {
        // DO NOT MERGE: Verify that A) this works on by computer and B) it will work on other
        // computers. I believe one or both of these may be wrong.
        // bytemuck::bytes_of(self)
        let mut value = [0u8; size_of::<Self>()];
        for i in 0..DIGEST_WORDS {
            // TODO: Check that BE is the right byte order.
            value[i..i + DIGEST_WORD_SIZE].copy_from_slice(&self.0.get()[i].to_be_bytes());
        }
        &value
    }
}

impl From<Digest> for Node {
    fn from(digest: Digest) -> Self {
        Self(digest)
    }
}

impl Element for Node {
    fn byte_len() -> usize {
        size_of::<Self>()
    }

    fn from_slice(bytes: &[u8]) -> Self {
        let mut words = [0u32; DIGEST_WORDS];
        for i in 0..DIGEST_WORDS {
            // TODO: Check that BE is the right byte order.
            words[i] = u32::from_be_bytes(
                bytes[i..i + DIGEST_WORD_SIZE]
                    .try_into()
                    .expect("conversion of bytes into a digest word failed"),
            );
        }
        // NOTE: Nicer if there was a way to construct a digest from words without copying.
        Self(Digest::from_slice(&words))
    }

    fn copy_to_slice(&self, bytes: &mut [u8]) {
        for i in 0..DIGEST_WORDS {
            // TODO: Check that BE is the right byte order.
            bytes[i..i + DIGEST_WORD_SIZE].copy_from_slice(&self.0.get()[i].to_be_bytes());
        }
    }
}

impl Distribution<Node> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Node {
        Node::from(Digest::from_slice(
            &rand::thread_rng().gen::<[u32; DIGEST_WORDS]>(),
        ))
    }
}

struct ShaHasher<H>
where
    H: Sha + 'static,
{
    data: Vec<u8>,
    sha: &'static H,
}

// NOTE: It would be nice if Sha structs (or the trait) implemented Default.
// Since it doesn't we need to impl default per struct implementation.
impl Default for ShaHasher<risc0_zkp::core::sha_cpu::Impl> {
    fn default() -> Self {
        Self {
            data: Vec::new(),
            sha: risc0_zkp::core::sha::default_implementation(),
        }
    }
}

impl<H: Sha> Hasher for ShaHasher<H> {
    // NOTE: RISC0 Sha trait only provides clean ways to hash data in one shot. As a result, we
    // append the data to an array here. This is fine for short messages.
    fn write(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }

    fn finish(&self) -> u64 {
        unimplemented!("finish is not implemented for merkletree hashers");
    }
}

impl<H: Sha> Algorithm<Node> for ShaHasher<H>
where
    ShaHasher<H>: Default,
{
    fn hash(&mut self) -> Node {
        // NOTE: Does Sha need to be a struct rather than a static method?
        Node(*self.sha.hash_bytes(&self.data))
    }
}

fn random_elements(elems: usize) -> Vec<Node> {
    (0..elems)
        .map(|_| rand::thread_rng().gen())
        .collect::<Vec<_>>()
}

fn main() -> Result<(), Box<dyn Error>> {
    let elements = random_elements(1 << 10);
    let tree = MerkleTree::new(elements)?;
    println!("Created a merkle tree with {} elements", tree.len());
    Ok(())
}
