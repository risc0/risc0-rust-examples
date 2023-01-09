use std::ops::Deref;

use image::DynamicImage;

use crate::merkle::MerkleTree;

/// Recommended default chunk size to use in the ImageMerkleTree and ImageOracle.
pub const IMAGE_CHUNK_SIZE: u32 = 8;

/// ImageMerkleTree is a merklization of an image, constructed with the leaf elements being NxN
/// square chunks.
///
/// Chunks on the right and bottom boundaries will be incomplete if the width or
/// height cannot be divided by N.
pub struct ImageMerkleTree<const N: u32>(MerkleTree<Vec<u8>>);

impl<const N: u32> ImageMerkleTree<N> {
    pub fn new(image: &DynamicImage) -> Self {
        // Iterate over the NxN chunks of an image in right to left, top to bottom, order.
        // Convert the image into RGB8 as it is chunked. Access to the image will be to the
        // underlying subpixels (i.e. bytes for RGB8).
        // NOTE: With support for batched Merkle proof generation and verification, it is likely
        // that this construction could be made more efficient by linearizing the chunks such that
        // chunks that are close in the image are close in the vector.
        let chunks: Vec<Vec<u8>> = {
            (0..image.height())
                .step_by(usize::try_from(N).unwrap())
                .map(|y| {
                    (0..image.width())
                        .step_by(usize::try_from(N).unwrap())
                        .map(move |x| image.crop_imm(x, y, N, N).into_rgb8().into_raw())
                })
                .flatten()
                .collect()
        };

        Self(MerkleTree::new(chunks))
    }
}

impl<const N: u32> Deref for ImageMerkleTree<N> {
    type Target = MerkleTree<Vec<u8>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// ImageOracle provides verified access to an image held by the host.
#[cfg(target_os = "zkvm")]
mod zkvm {
    use divrem::{DivCeil, DivRem};
    use elsa::FrozenBTreeMap;
    use image::{GenericImageView, Rgb};

    use crate::merkle::{Node, VectorOracle};

    pub struct ImageOracle<const N: u32> {
        chunks: VectorOracle<Vec<u8>>,

        // Width and height of the image in pixels.
        width: u32,
        height: u32,

        // Fields used internally for precomputation and caching.
        width_chunks: u32,
        cache: FrozenBTreeMap<(u32, u32), Vec<u8>>,
    }

    impl<const N: u32> ImageOracle<N> {
        pub fn new(root: Node, width: u32, height: u32) -> Self {
            Self {
                chunks: VectorOracle::<Vec<u8>>::new(root),
                width,
                height,
                width_chunks: DivCeil::div_ceil(width, N),
                cache: Default::default(),
            }
        }

        /// Memoized method for getting chunks of the image. Inputs x and y are chunk coordinates.
        pub fn get_chunk(&self, x: u32, y: u32) -> &[u8] {
            // Check that the given x  if within the bounds of the width. No need to check y since
            // if y is out of bounds the VectorOrcacle query will be out of bounds.
            match self.cache.get(&(x, y)) {
                Some(chunk) => chunk,
                None => {
                    assert!(x < self.width_chunks);

                    let chunk = self
                        .chunks
                        .get(usize::try_from(y * self.width_chunks + x).unwrap());
                    self.cache.insert((x, y), chunk);
                    self.cache.get(&(x, y)).unwrap()
                }
            }
        }

        pub fn root(&self) -> &Node {
            self.chunks.root()
        }
    }

    impl<const N: u32> GenericImageView for ImageOracle<N> {
        type Pixel = Rgb<u8>;

        fn dimensions(&self) -> (u32, u32) {
            (self.width, self.height)
        }

        fn bounds(&self) -> (u32, u32, u32, u32) {
            (0, 0, self.width, self.height)
        }

        #[inline(always)]
        fn get_pixel(&self, x: u32, y: u32) -> Self::Pixel {
            assert!(self.in_bounds(x, y));

            // Calculate split x and y into the chunk selector portion and offset.
            let (x_chunk, x_offset) = DivRem::div_rem(x, N);
            let (y_chunk, y_offset) = DivRem::div_rem(y, N);

            let chunk = &self.get_chunk(x_chunk, y_chunk);

            // FIXME: Does not handle access at the edge of the image.
            <[u8; 3]>::try_from(
                &chunk[usize::try_from(y_offset * N + x_offset).unwrap() * 3..][..3],
            )
            .unwrap()
            .into()
        }
    }
}

#[cfg(target_os = "zkvm")]
pub use crate::image::zkvm::*;
