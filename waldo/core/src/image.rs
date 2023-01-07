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
    use divrem::{DivCeil, DivFloor, DivRem};
    use image::{GenericImageView, Rgb};

    use crate::merkle::{Node, VectorOracle};

    pub struct ImageOracle<const N: u32> {
        chunks: VectorOracle<Vec<u8>>,
        width: u32,
        height: u32,

        // Fields related to the loaded subimage.
        // TODO: Rename fields
        cache_chunk_x_min: u32,
        cache_chunk_x_max: u32,
        cache_chunk_width: u32,
        cache_chunk_y_min: u32,
        cache_chunk_y_max: u32,
        cache_chunk_height: u32,
        cache_chunks: Vec<Vec<u8>>,
    }

    #[cfg(target_os = "zkvm")]
    impl<const N: u32> ImageOracle<N> {
        pub fn new(root: Node, width: u32, height: u32) -> Self {
            Self {
                chunks: VectorOracle::<Vec<u8>>::new(root),
                width,
                height,
                cache_chunk_x_min: 0,
                cache_chunk_x_max: 0,
                cache_chunk_width: 0,
                cache_chunk_y_min: 0,
                cache_chunk_y_max: 0,
                cache_chunk_height: 0,
                cache_chunks: Vec::new(),
            }
        }

        /// Load a bounded rectangular crop of the image into memory from the host. It is only possible
        /// to operate over a small portion of the image within the guest without running out of
        /// cycles. This function declares what portion of the image will be operated over.
        /// TODO: See if this can be implemented reasonably without the need to preload a bounded set
        /// of chunks.
        pub fn load_crop(&mut self, x: u32, y: u32, width: u32, height: u32) {
            assert!(self.in_bounds(x + width, y + height));

            // Crop buffer needs to be large enough to hold the chunks to be loaded. Each chunk is
            // overlapped by the crop area will need to be loaded.
            self.cache_chunk_x_min = DivFloor::div_floor(x, N);
            self.cache_chunk_x_max = DivCeil::div_ceil(x + width, N);
            self.cache_chunk_y_min = DivFloor::div_floor(y, N);
            self.cache_chunk_y_max = DivCeil::div_ceil(y + height, N);
            self.cache_chunk_width = self.cache_chunk_x_max - self.cache_chunk_x_min;
            self.cache_chunk_height = self.cache_chunk_y_max - self.cache_chunk_y_min;
            self.cache_chunks = Vec::with_capacity(
                usize::try_from(self.cache_chunk_width * self.cache_chunk_height).unwrap(),
            );

            // Load into the struct buffer all chunks overlapped by the indicated rectangle.
            let image_width_chunks = DivCeil::div_ceil(self.width, N);
            for y_chunk in self.cache_chunk_y_min..self.cache_chunk_y_max {
                for x_chunk in self.cache_chunk_x_min..self.cache_chunk_x_max {
                    self.cache_chunks.push(
                        self.chunks
                            .get(usize::try_from(y_chunk * image_width_chunks + x_chunk).unwrap()),
                    );
                }
            }
        }

        pub fn root(&self) -> &Node {
            self.chunks.root()
        }
    }

    #[cfg(target_os = "zkvm")]
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
            // Calculate split x and y into the chunk selector portion and offset.
            let (x_chunk, x_offset) = DivRem::div_rem(x, N);
            let (y_chunk, y_offset) = DivRem::div_rem(y, N);

            // Check that the pixel location is in bounds. Only need to check width since overruning
            // the height will result in a panic due to out of bounds slice indexing.
            if x_chunk < self.cache_chunk_x_min || x_chunk >= self.cache_chunk_x_max {
                panic!(
                    "access out of loaded image bound: {:?} on {}x{} image with loaded bounds {:?}",
                    (x, y),
                    self.width,
                    self.height,
                    (
                        self.cache_chunk_x_min * N,
                        self.cache_chunk_x_max * N,
                        self.cache_chunk_y_min * N,
                        self.cache_chunk_y_max * N,
                    ),
                );
            }

            let chunk = &self.cache_chunks[usize::try_from(
                y_chunk.checked_sub(self.cache_chunk_y_min).unwrap() * self.cache_chunk_width
                    + x_chunk.checked_sub(self.cache_chunk_x_min).unwrap(),
            )
            .unwrap()];

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
