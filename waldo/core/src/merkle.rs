use std::cmp::Ordering;
use std::hash::Hasher;
use std::marker::PhantomData;
use std::mem::size_of;
use std::ops::Deref;

use bytemuck::{Pod, Zeroable};
use merkle_light::hash::{Algorithm, Hashable};
use merkle_light::{merkle, proof};
use risc0_zkp::core::sha::{Digest, Sha, DIGEST_WORDS, DIGEST_WORD_SIZE};
use risc0_zkp::core::sha_cpu;
#[cfg(feature = "zkvm")]
use risc0_zkvm_guest as guest;
use serde::{Deserialize, Serialize};

// Pick the appropriate implementation of SHA2-256 depending on whether we are in the zkVM guest.
cfg_if::cfg_if! {
    if #[cfg(all(target_os = "zkvm", feature = "zkvm"))] {
        pub type ShaImpl = guest::sha::Impl;
    } else {
        pub type ShaImpl = sha_cpu::Impl;
    }
}

/// Merkle tree for use as a vector commitment over elements of the specified type.
pub struct MerkleTree<Element>
where
    Element: Hashable<ShaHasher<ShaImpl>>,
{
    inner: merkle::MerkleTree<Node, ShaHasher<ShaImpl>>,
    phantom_elem: PhantomData<Element>,
}

impl<Element> MerkleTree<Element>
where
    Element: Hashable<ShaHasher<ShaImpl>>,
{
    pub fn prove(&self, i: usize) -> Proof<Element> {
        self.gen_proof(i).into()
    }
}

impl<Element> Deref for MerkleTree<Element>
where
    Element: Hashable<ShaHasher<ShaImpl>>,
{
    type Target = merkle::MerkleTree<Node, ShaHasher<ShaImpl>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<Element> From<merkle::MerkleTree<Node, ShaHasher<ShaImpl>>> for MerkleTree<Element>
where
    Element: Hashable<ShaHasher<ShaImpl>>,
{
    fn from(inner: merkle::MerkleTree<Node, ShaHasher<ShaImpl>>) -> Self {
        Self {
            inner,
            phantom_elem: PhantomData,
        }
    }
}

pub struct Proof<Element>
where
    Element: Hashable<ShaHasher<ShaImpl>>,
{
    inner: proof::Proof<Node>,
    phantom_elem: PhantomData<Element>,
}

impl<Element> Proof<Element>
where
    Element: Hashable<ShaHasher<ShaImpl>>,
{
    pub fn verify(&self, root: &Node, element: &Element) -> bool {
        // Check that the root of the proof matches the provided root.
        // TOOD: Is this the best way of doing this? It requires the user to provide a root, which
        // avoids the sharp edge of forgetting to check against a fixed root, but may be less
        // flexible than it could be.
        if &self.root() != root {
            return false;
        }

        // Check that the path from the leaf matches the root.
        if !self.validate::<ShaHasher<ShaImpl>>() {
            return false;
        }

        // Check the element hashes to the leaf in the proof.
        // Hash the element.
        let algorithm = &mut ShaHasher::<ShaImpl>::default();
        element.hash(algorithm);
        let elem_hash = algorithm.hash();

        // Hash the hash of the  element to get the leaf.
        algorithm.reset();
        let leaf_hash = algorithm.leaf(elem_hash);

        leaf_hash == self.item()
    }
}

impl<Element> Deref for Proof<Element>
where
    Element: Hashable<ShaHasher<ShaImpl>>,
{
    type Target = proof::Proof<Node>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<Element> From<proof::Proof<Node>> for Proof<Element>
where
    Element: Hashable<ShaHasher<ShaImpl>>,
{
    fn from(inner: proof::Proof<Node>) -> Self {
        Self {
            inner,
            phantom_elem: PhantomData,
        }
    }
}

/// Wrapper on the RISC0 Digest type to allow it to act as a Merkle tree element.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Pod, Zeroable, Deserialize, Serialize)]
#[repr(transparent)]
pub struct Node(Digest);

const_assert_eq!(size_of::<Node>(), DIGEST_WORDS * DIGEST_WORD_SIZE);

/// Node is a wrapper around the RISC0 SHA2-256 digest type with the needed trait inmplementations
/// to be used as a node in the merkle_light package.
impl Node {
    // Constructs the byte array digest value from big endian representation of the u32 words.
    // NOTE: I tested this on my (little endian) x86 machine. Have not tested it on a big endian
    // machine.
    pub fn to_be_bytes(&self) -> [u8; size_of::<Self>()] {
        let mut value = [0u8; size_of::<Self>()];
        for i in 0..DIGEST_WORDS {
            value[i * DIGEST_WORD_SIZE..(i + 1) * DIGEST_WORD_SIZE]
                .copy_from_slice(&self.0.get()[i].to_be_bytes());
        }
        value
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

/// ShaHasher is a wrapper around the RISC0 SHA2-256 implementations that implements the Algorithm
/// trait for use with the merkle_light package.
pub struct ShaHasher<H>
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

#[cfg(feature = "zkvm")]
static ZKVM_SHA_IMPL: &'static guest::sha::Impl = &guest::sha::Impl {};

#[cfg(feature = "zkvm")]
impl Default for ShaHasher<guest::sha::Impl> {
    fn default() -> Self {
        Self {
            data: Vec::new(),
            sha: ZKVM_SHA_IMPL,
        }
    }
}

// NOTE: The Hasher trait is really designed for use with hashmaps and is quite ill-suited as an
// interface for use by merkle_light. This is one of the design weaknesses of this package.
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic_merkle_tree_constuction_works() {
        let items = (0..1 << 10).collect::<Vec<_>>();
        let tree = MerkleTree::<u32>::from(merkle::MerkleTree::<_, ShaHasher<ShaImpl>>::from_data(
            &items,
        ));
        assert_eq!(tree.len(), 2047);

        let proof = tree.prove(47);
        assert!(proof.verify(&tree.root(), &47));
    }

    #[test]
    fn algorithm_is_consistent_with_sha2() {
        let test_string: &'static str = "RISCO SHA hasher test string";
        let mut r0_hasher = ShaHasher::<ShaImpl>::default();
        r0_hasher.write(test_string.as_bytes());
        let r0_node = r0_hasher.hash();

        use sha2::Digest;
        let mut rc_hasher = sha2::Sha256::new();
        rc_hasher.update(test_string);
        let rc_hash: &[u8] = &rc_hasher.finalize()[..];

        // NOTE: This checks against the big endian representation of the digest, which is not what
        // is used by AsRef and therefore is also not what is used in the tree.
        assert_eq!(hex::encode(r0_node.to_be_bytes()), hex::encode(rc_hash));
    }
}
