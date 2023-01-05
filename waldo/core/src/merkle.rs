use std::cmp::Ordering;
use std::hash::Hasher;
use std::marker::PhantomData;
use std::mem::size_of;
use std::ops::Deref;

use bytemuck::{Pod, Zeroable};
use merkle_light::hash::{Algorithm, Hashable};
use merkle_light::{merkle, proof};
use risc0_zkp::core::sha::{Digest, Sha, DIGEST_WORDS, DIGEST_WORD_SIZE};
#[cfg(not(target_os = "zkvm"))]
use risc0_zkp::core::sha_cpu;
#[cfg(target_os = "zkvm")]
use risc0_zkvm::guest;
use serde::{Deserialize, Serialize};

/// RISC0 channel identifier for providing oracle access to a vector to the guest from the host.
pub const VECTOR_ORACLE_CHANNEL: u32 = 0x09ac1e00;

// Pick the appropriate implementation of SHA2-256 depending on whether we are in the zkVM guest.
cfg_if::cfg_if! {
    if #[cfg(target_os = "zkvm")] {
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
    tree: merkle::MerkleTree<Node, ShaHasher<ShaImpl>>,
    elements: Vec<Element>,
}

impl<Element> MerkleTree<Element>
where
    Element: Hashable<ShaHasher<ShaImpl>>,
{
    pub fn new(elements: Vec<Element>) -> Self {
        Self {
            tree: merkle::MerkleTree::<_, ShaHasher<ShaImpl>>::from_data(elements.iter()),
            elements,
        }
    }

    pub fn elements(&self) -> &[Element] {
        &self.elements
    }

    pub fn prove(&self, i: usize) -> Proof<Element> {
        self.tree.gen_proof(i).into()
    }
}

#[cfg(not(target_os = "zkvm"))]
impl<Element> MerkleTree<Element>
where
    Element: Hashable<ShaHasher<ShaImpl>>
        + Serialize
        // Debug
        + std::fmt::Debug
        + Default
        + std::cmp::PartialEq,
{
    pub fn vector_oracle_callback<'a>(&'a self) -> impl Fn(u32, &[u8]) -> Vec<u8> + 'a {
        |channel_id, data| {
            // Callback function must only be registered as a callback for the VECTOR_ORACLE_CHANNEL.
            assert_eq!(channel_id, VECTOR_ORACLE_CHANNEL);
            // NOTE: This would be nicer with we could avoid bytemuck.
            let index: usize = bincode::deserialize::<u32>(data)
                .unwrap()
                .try_into()
                .unwrap();
            let value = &self.elements()[index];

            let proof = self.prove(index);

            // Debug
            assert!(proof.verify(&self.root(), &value));
            bincode::serialize(&(value, proof)).unwrap()
        }
    }
}

impl<Element> Deref for MerkleTree<Element>
where
    Element: Hashable<ShaHasher<ShaImpl>>,
{
    type Target = merkle::MerkleTree<Node, ShaHasher<ShaImpl>>;

    fn deref(&self) -> &Self::Target {
        &self.tree
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(from = "(Vec<Node>, Vec<bool>)", into = "(Vec<Node>, Vec<bool>)")]
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

        // DEBUG
        //#[cfg(target_os = "zkvm")]
        //{
        //    assert_eq!(format!("root {:x?}", root), "")
        //};
        if &self.inner.root() != root {
            assert!(false);
            return false;
        }

        // Check that the path from the leaf matches the root.
        if !self.inner.validate::<ShaHasher<ShaImpl>>() {
            assert!(false);
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

        assert!(leaf_hash == self.inner.item());
        leaf_hash == self.inner.item()
    }

    // Index computes, from the path, the index of the proven element in the vector.
    pub fn index(&self) -> usize {
        self.inner
            .path()
            .iter()
            .rfold(0, |index, bit| (index << 1) + (!*bit as usize))
    }
}

impl<Element> Clone for Proof<Element>
where
    Element: Hashable<ShaHasher<ShaImpl>>,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            phantom_elem: PhantomData,
        }
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

impl<Element> From<(Vec<Node>, Vec<bool>)> for Proof<Element>
where
    Element: Hashable<ShaHasher<ShaImpl>>,
{
    fn from(tuple: (Vec<Node>, Vec<bool>)) -> Self {
        proof::Proof::new(tuple.0, tuple.1).into()
    }
}

impl<Element> Into<(Vec<Node>, Vec<bool>)> for Proof<Element>
where
    Element: Hashable<ShaHasher<ShaImpl>>,
{
    fn into(self) -> (Vec<Node>, Vec<bool>) {
        (self.inner.lemma().to_vec(), self.inner.path().to_vec())
    }
}

/// Wrapper on the RISC0 Digest type to allow it to act as a Merkle tree element.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Pod, Zeroable, Deserialize, Serialize)]
// #[serde(from = "[u8; NODE_SIZE]", into = "[u8; NODE_SIZE]")]
#[repr(transparent)]
pub struct Node(Digest);

const NODE_SIZE: usize = DIGEST_WORDS * DIGEST_WORD_SIZE;
const_assert_eq!(size_of::<Node>(), NODE_SIZE);

/// Node is a wrapper around the RISC0 SHA2-256 digest type with the needed trait inmplementations
/// to be used as a node in the merkle_light package.
impl Node {
    // Constructs the byte array digest value from big endian representation of the u32 words.
    // NOTE: I tested this on my (little endian) x86 machine. Have not tested it on a big endian
    // machine.
    pub fn to_be_bytes(&self) -> [u8; NODE_SIZE] {
        let mut value = [0u8; NODE_SIZE];
        for i in 0..DIGEST_WORDS {
            value[i * DIGEST_WORD_SIZE..(i + 1) * DIGEST_WORD_SIZE]
                .copy_from_slice(&self.0.get()[i].to_be_bytes());
        }
        value
    }

    pub fn from_be_bytes(bytes: [u8; NODE_SIZE]) -> Self {
        let mut value = [0u32; DIGEST_WORDS];
        for i in 0..DIGEST_WORDS {
            value[i] = u32::from_be_bytes(
                bytes[i * DIGEST_WORD_SIZE..(i + 1) * DIGEST_WORD_SIZE]
                    .try_into()
                    .unwrap(),
            );
        }
        Self::from(Digest::new(value))
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

impl From<[u8; NODE_SIZE]> for Node {
    fn from(bytes: [u8; NODE_SIZE]) -> Self {
        #[cfg(target_os = "zkvm")]
        {
            Self::from_be_bytes(bytes)
        }

        #[cfg(not(target_os = "zkvm"))]
        {
            bytemuck::cast(bytes)
        }
    }
}

impl Into<[u8; NODE_SIZE]> for Node {
    fn into(self) -> [u8; NODE_SIZE] {
        #[cfg(target_os = "zkvm")]
        {
            self.to_be_bytes()
        }

        #[cfg(not(target_os = "zkvm"))]
        {
            bytemuck::cast(self)
        }
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

#[cfg(not(target_os = "zkvm"))]
static CPU_SHA_IMPL: &'static sha_cpu::Impl = &sha_cpu::Impl {};

// NOTE: It would be nice if Sha structs (or the trait) implemented Default.
// Since it doesn't we need to impl default per struct implementation.
#[cfg(not(target_os = "zkvm"))]
impl Default for ShaHasher<sha_cpu::Impl> {
    fn default() -> Self {
        Self {
            data: Vec::new(),
            sha: CPU_SHA_IMPL,
        }
    }
}

#[cfg(target_os = "zkvm")]
static ZKVM_SHA_IMPL: &'static guest::sha::Impl = &guest::sha::Impl {};

#[cfg(target_os = "zkvm")]
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
        let node = Node::from(*self.sha.hash_bytes(&self.data));
        // println!("ShaHasher(data: {:x?}).hash() = {:x?}", &self.data, &node);
        node
    }
}

#[cfg(target_os = "zkvm")]
pub struct VectorOracle<Element>
where
    Element: Hashable<ShaHasher<ShaImpl>> + Deserialize<'static>,
{
    root: Node,
    phantom_elem: PhantomData<Element>,
}

#[cfg(target_os = "zkvm")]
impl<Element> VectorOracle<Element>
where
    Element: Hashable<ShaHasher<ShaImpl>>
        + Deserialize<'static>
        // Debug
        + std::fmt::Debug
        + Default
        + std::cmp::PartialEq,
{
    pub fn new(root: Node) -> Self {
        Self {
            root,
            phantom_elem: PhantomData,
        }
    }

    // NOTE: VectorOracle does not attempt to verify the length of the committed vector, or that
    // there is a valid, known, element at every index. Any out of bounds access or access to an
    // index for which there is no element will not return since no valid proof can be generated.
    // NOTE: It would be better to use the tailored risc0_zeroio crate for this instead of serde.
    pub fn get(&self, index: usize) -> Element {
        let (value, proof): (Element, Proof<Element>) = bincode::deserialize(
            // Fetch the value and proof from the host by index.
            // NOTE: It would be nice if there was a wrapper for send_recv that looked more like
            // env::read(). A smaller step would be to have this method as take [u32] instead of [u8]
            // to avoid mucking around with the bytes.
            // TODO: Consider using bincode or another byte serializer instead of the u32 RISC0 format.
            guest::env::send_recv(
                VECTOR_ORACLE_CHANNEL,
                // Cast to u32 before serializing since usize is an architecture dependent type.
                &bincode::serialize(&(u32::try_from(index).unwrap())).unwrap(),
            ),
        )
        .unwrap();

        // Verify the proof for the value of the element at the given index in the committed vector.
        assert_eq!(index, proof.index());
        assert!(proof.verify(&self.root, &value));
        value
    }

    pub fn root(&self) -> &Node {
        &self.root
    }
}

#[cfg(test)]
mod test {
    use rand::Rng;

    use super::*;

    /// Build and return a random Merkle tree with 1028 u32 elements.
    fn random_merkle_tree() -> MerkleTree<u32> {
        let item_count: usize = rand::thread_rng().gen_range((1 << 10)..(1 << 12));
        let items: Vec<u32> = (0..item_count).map(|_| rand::thread_rng().gen()).collect();
        MerkleTree::<u32>::new(items)
    }

    #[test]
    fn merkle_tree_proving_works() {
        let tree = random_merkle_tree();
        for (index, item) in tree.elements().iter().enumerate() {
            let proof = tree.prove(index);
            assert!(proof.verify(&tree.root(), &item));
        }
    }

    #[test]
    fn merkle_proof_serialization_works() {
        let tree = random_merkle_tree();
        for (index, item) in tree.elements().iter().enumerate() {
            let proof = tree.prove(index);

            let proof_bytes = bincode::serialize(&proof).unwrap();
            let proof_deserialized: Proof<u32> = bincode::deserialize(&proof_bytes).unwrap();

            assert!(proof_deserialized.verify(&tree.root(), &item));
        }
    }

    #[test]
    fn merkle_proof_index_works() {
        let tree = random_merkle_tree();
        for (index, _item) in tree.elements().iter().enumerate() {
            let proof = tree.prove(index);
            assert_eq!(proof.index(), index);
        }
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
        assert_eq!(hex::encode(r0_node), hex::encode(rc_hash));
    }
}
