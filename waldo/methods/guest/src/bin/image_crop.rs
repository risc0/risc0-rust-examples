#![no_main]
// #![no_std]

use divrem::{DivCeil, DivFloor, DivRem};
use image::{imageops, GenericImageView, Rgb};
use risc0_zkvm::guest::env;
use waldo_core::merkle::{Node, VectorOracle};
use waldo_core::{Journal, PrivateInput, IMAGE_CHUNK_SIZE};

risc0_zkvm::guest::entry!(main);

/// ImageOracle provides verified access to an image held by the host.
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
        self.cache_chunks = Vec::with_capacity(usize::try_from(self.cache_chunk_width * self.cache_chunk_height).unwrap());

        // Load into the struct buffer all chunks overlapped by the indicated rectangle.
        let image_width_chunks = DivCeil::div_ceil(self.width, N);
        for y_chunk in self.cache_chunk_y_min..self.cache_chunk_y_max {
            for x_chunk in self.cache_chunk_x_min..self.cache_chunk_x_max {
                self.cache_chunks.push(
                    self
                        .chunks
                        .get(usize::try_from(y_chunk * image_width_chunks + x_chunk).unwrap()),
                );
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

        let chunk = &self.cache_chunks[usize::try_from(y_chunk.checked_sub(self.cache_chunk_y_min).unwrap() * self.cache_chunk_width + x_chunk.checked_sub(self.cache_chunk_x_min).unwrap()).unwrap()];

        // FIXME: Does not handle access at the edge of the image.
        <[u8; 3]>::try_from(
            &chunk[usize::try_from(y_offset * N + x_offset).unwrap() * 3..][..3],
        )
        .unwrap()
        .into()
    }
}

pub fn main() {
    // Read a Merkle proof from the host.
    let input: PrivateInput = env::read();

    // Initialize a Merkle tree based vector oracle, supporting verified access to a vector of data
    // on the host. Use the oracle to access a range of elements from the host.
    let mut oracle = ImageOracle::<{IMAGE_CHUNK_SIZE}>::new(
        input.root,
        input.image_dimensions.0,
        input.image_dimensions.1,
    );

    // TODO: See if there is a better way to factor this.
    oracle.load_crop(
        input.crop_location.0,
        input.crop_location.1,
        input.crop_dimensions.0,
        input.crop_dimensions.1,
    );

    let crop = imageops::crop_imm(
        &oracle,
        input.crop_location.0,
        input.crop_location.1,
        input.crop_dimensions.0,
        input.crop_dimensions.1,
    );

    // Collect the verified public information into the journal.
    let journal = Journal {
        root: *oracle.root(),
        image_dimensions: oracle.dimensions(),
        subimage: crop.to_image().into_raw(),
        subimage_dimensions: input.crop_dimensions,
    };
    env::commit(&journal);
}
