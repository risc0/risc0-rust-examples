use std::cmp::Ordering;
use std::hash::Hasher;
use std::marker::PhantomData;
use std::ops::Deref;

use bytemuck::{Pod, Zeroable};
use merkle_light::hash::{Algorithm, Hashable};
use merkle_light::{merkle, proof};
use risc0_zkp::core::sha::{Digest, Sha};
#[cfg(target_os = "zkvm")]
use risc0_zkvm::guest;
use risc0_zkvm::sha::sha;
use serde::{Deserialize, Serialize};

/// RISC0 channel identifier for providing oracle access to a vector to the guest from the host.
pub const VECTOR_ORACLE_CHANNEL: u32 = 0x09ac1e00;

/// Merkle tree for use as a vector commitment over elements of the specified type.
pub struct MerkleTree<Element>
where
    Element: Hashable<ShaHasher>,
{
    tree: merkle::MerkleTree<Node, ShaHasher>,
    elements: Vec<Element>,
}

impl<Element> MerkleTree<Element>
where
    Element: Hashable<ShaHasher>,
{
    pub fn new(elements: Vec<Element>) -> Self {
        Self {
            tree: merkle::MerkleTree::<_, ShaHasher>::from_data(elements.iter()),
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
    Element: Hashable<ShaHasher> + Serialize,
{
    pub fn vector_oracle_callback<'a>(&'a self) -> impl Fn(u32, &[u8]) -> Vec<u8> + 'a {
        |channel_id, data| {
            // Callback function must only be registered as a callback for the VECTOR_ORACLE_CHANNEL.
            assert_eq!(channel_id, VECTOR_ORACLE_CHANNEL);
            // TODO: Using bincode here, but it would likely be better on the guest side to use the
            // risc0 zeroio or serde creates. I should try to use one of those (again).
            let index: usize = bincode::deserialize::<u32>(data)
                .unwrap()
                .try_into()
                .unwrap();

            let value = &self.elements()[index];
            let proof = self.prove(index);

            assert!(proof.verify(&self.root(), &value));
            bincode::serialize(&(value, proof)).unwrap()
        }
    }
}

// Implement Deref to the merkle_light tree so that all the methods on that type accessible.
impl<Element> Deref for MerkleTree<Element>
where
    Element: Hashable<ShaHasher>,
{
    type Target = merkle::MerkleTree<Node, ShaHasher>;

    fn deref(&self) -> &Self::Target {
        &self.tree
    }
}

/// Wrapper for the merkle_light inclusion proof. Includes and improved API for verifying that a
/// proof references and expected element and position. Supports serialization.
#[derive(Debug, Serialize, Deserialize)]
#[serde(from = "(Vec<Node>, Vec<bool>)", into = "(Vec<Node>, Vec<bool>)")]
pub struct Proof<Element>
where
    Element: Hashable<ShaHasher>,
{
    inner: proof::Proof<Node>,
    phantom_elem: PhantomData<Element>,
}

impl<Element> Proof<Element>
where
    Element: Hashable<ShaHasher>,
{
    pub fn verify(&self, root: &Node, element: &Element) -> bool {
        // Check that the root of the proof matches the provided root.
        match &self.verified_root(element) {
            Some(ref verified_root) => verified_root == root,
            None => false,
        }
    }

    pub fn verified_root(&self, element: &Element) -> Option<Node> {
        // Check that the path from the leaf to the root it consistent.
        if !self.inner.validate::<ShaHasher>() {
            return None;
        }

        // Check the element hashes to the leaf in the proof.
        let algorithm = &mut ShaHasher::default();
        element.hash(algorithm);
        let elem_hash = algorithm.hash();

        // Hash the hash of the element to get the leaf, and check that it matches.
        algorithm.reset();
        if algorithm.leaf(elem_hash) != self.inner.item() {
            return None;
        }

        Some(self.root())
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
    Element: Hashable<ShaHasher>,
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
    Element: Hashable<ShaHasher>,
{
    type Target = proof::Proof<Node>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<Element> From<proof::Proof<Node>> for Proof<Element>
where
    Element: Hashable<ShaHasher>,
{
    fn from(inner: proof::Proof<Node>) -> Self {
        Self {
            inner,
            phantom_elem: PhantomData,
        }
    }
}

// From tuple representation provided to enable serde deserialization.
impl<Element> From<(Vec<Node>, Vec<bool>)> for Proof<Element>
where
    Element: Hashable<ShaHasher>,
{
    fn from(tuple: (Vec<Node>, Vec<bool>)) -> Self {
        proof::Proof::new(tuple.0, tuple.1).into()
    }
}

// From tuple representation provided to enable serde deserialization.
impl<Element> Into<(Vec<Node>, Vec<bool>)> for Proof<Element>
where
    Element: Hashable<ShaHasher>,
{
    fn into(self) -> (Vec<Node>, Vec<bool>) {
        (self.inner.lemma().to_vec(), self.inner.path().to_vec())
    }
}

/// Wrapper on the RISC0 Digest type to allow it to act as a merkle_light Element.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Pod, Zeroable, Deserialize, Serialize)]
#[repr(transparent)]
pub struct Node(Digest);

impl AsRef<[u8]> for Node {
    fn as_ref(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

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
#[derive(Default)]
pub struct ShaHasher {
    data: Vec<u8>,
}

// NOTE: The Hasher trait is really designed for use with hashmaps and is quite ill-suited as an
// interface for use by merkle_light. This is one of the design weaknesses of this package.
impl Hasher for ShaHasher {
    // NOTE: RISC0 Sha trait only provides clean ways to hash data in one shot. As a result, we
    // append the data to an array here.
    fn write(&mut self, bytes: &[u8]) {
        self.data.extend_from_slice(bytes);
    }

    fn finish(&self) -> u64 {
        unimplemented!("finish is not implemented for merkletree hashers");
    }
}

impl Algorithm<Node> for ShaHasher {
    fn hash(&mut self) -> Node {
        Node::from(*sha().hash_bytes(&self.data))
    }
}

#[cfg(target_os = "zkvm")]
pub struct VectorOracle<Element>
where
    Element: Hashable<ShaHasher> + Deserialize<'static>,
{
    root: Node,
    phantom_elem: PhantomData<Element>,
}

// TODO: Provide a version of this primative that is designed to provide oracle access to bytes as
// &'static [u8] references instead of serializing and deserializing vectors.
#[cfg(target_os = "zkvm")]
impl<Element> VectorOracle<Element>
where
    Element: Hashable<ShaHasher> + Deserialize<'static>,
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
    use sha2::Digest as Sha2Digest;

    use super::*;

    // NOTE: There are no tests for the VectorOracle functionalities. This is because it's somewhat
    // non-trivial to test this functionality. TODO is to implement these tests and consider how it
    // could be made easy for other developers.

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
        let test_string: &'static [u8] = "RISCO SHA hasher test string".as_bytes();
        let mut hasher = ShaHasher::default();
        hasher.write(test_string);
        let node = hasher.hash();

        let reference_hash = sha2::Sha256::digest(test_string);
        assert_eq!(hex::encode(node), hex::encode(reference_hash));
    }
}
