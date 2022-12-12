#[macro_use]
extern crate static_assertions;

use std::cmp::Ordering;
use std::error::Error;
use std::hash::Hasher;
use std::mem::size_of;

use bytemuck::{Pod, Zeroable};
use merkle_light::hash::Algorithm;
use merkle_light::merkle::MerkleTree;
use rand::distributions::{Distribution, Standard};
use rand::Rng;
use risc0_zkp::core::sha::{Digest, Sha, DIGEST_WORDS, DIGEST_WORD_SIZE};
use risc0_zkp::core::sha_cpu;

// Wrapper on the RISC0 Digest type to allow it to act as a Merkle tree element.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Pod, Zeroable)]
#[repr(transparent)]
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
        // NOTE: On Intel x86_64, this results in a value that does not match the canoncial
        // SHA2-256 hash function. If the u32 values were to be stored in big endian format, this
        // would match. See below for an example.
        bytemuck::bytes_of(self)
    }
}

impl Node {
    // Constructs the byte array digest value from big endian representation of the u32 words.
    // NOTE: I tested this on my (little endian) x86 machine. Have not tested it on a big endian
    // machine.
    fn to_be_bytes(&self) -> [u8; size_of::<Self>()] {
        let mut value = [0u8; size_of::<Self>()];
        for i in 0..DIGEST_WORDS {
            value[i * DIGEST_WORD_SIZE..(i + 1) * DIGEST_WORD_SIZE]
                .copy_from_slice(&self.0.get()[i].to_be_bytes());
        }
        value
    }
}

impl From<Digest> for Node {
    fn from(digest: Digest) -> Self {
        Self(digest)
    }
}

impl Into<Digest> for Node {
    fn into(self) -> Digest {
        self.0
    }
}

// Enable the random generation of nodes for testing an development.
impl Distribution<Node> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Node {
        Node::from(Digest::from_slice(&rng.gen::<[u32; DIGEST_WORDS]>()))
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
impl Default for ShaHasher<sha_cpu::Impl> {
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
    println!("===== construct a Merkle tree =====");
    let elements = random_elements(1 << 10);
    let tree = MerkleTree::<_, ShaHasher<sha_cpu::Impl>>::new(elements);
    println!("Created a merkle tree with {} elements", tree.len());
    println!("");

    println!("===== check consistency of r0 sha2 impl with RustCrypto =====");
    let test_string: &'static str = "RISCO SHA hasher test string";
    let mut r0_hasher = ShaHasher::<sha_cpu::Impl>::default();
    r0_hasher.write(test_string.as_bytes());
    let r0_node = r0_hasher.hash();
    let r0_hash: &[u8] = r0_node.as_ref();

    use sha2::Digest;
    let mut rc_hasher = sha2::Sha256::new();
    rc_hasher.update(test_string);
    let rc_hash: &[u8] = &rc_hasher.finalize()[..];

    println!("r0_hash       : {}", hex::encode(r0_hash));
    println!("r0_hash as be : {}", hex::encode(r0_node.to_be_bytes()));
    println!("rc_hash       : {}", hex::encode(rc_hash));

    Ok(())
}
