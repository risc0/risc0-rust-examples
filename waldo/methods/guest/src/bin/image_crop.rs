#![no_main]
// #![no_std]

use divrem::{DivCeil, DivFloor};
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
    crop_x: u32,
    crop_y: u32,
    crop_width: u32,
    crop_height: u32,
    crop_buffer: Vec<u8>,
}

impl<const N: u32> ImageOracle<N> {
    pub fn new(root: Node, width: u32, height: u32) -> Self {
        Self {
            chunks: VectorOracle::<Vec<u8>>::new(root),
            width,
            height,
            crop_x: 0,
            crop_y: 0,
            crop_width: 0,
            crop_height: 0,
            crop_buffer: Vec::new(),
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
        let overlap_width = (DivCeil::div_ceil(x + width, N) - DivFloor::div_floor(x, N)) * N;
        let overlap_height = (DivCeil::div_ceil(y + height, N) - DivFloor::div_floor(y, N)) * N;
        self.crop_buffer = Vec::with_capacity(usize::try_from(overlap_width * overlap_height * 3 ).unwrap());

        // Load into the struct buffer all chunks overlapped by the indicated rectangle.
        let width_chunks = DivCeil::div_ceil(self.width, N);
        for y_chunk in (y / N)..DivCeil::div_ceil(y + height, N) {
            for x_chunk in (x / N)..DivCeil::div_ceil(x + width, N) {
                self.crop_buffer.extend_from_slice(
                    &self
                        .chunks
                        .get(usize::try_from(y_chunk * width_chunks + x_chunk).unwrap()),
                );
            }
        }

        // Record the bounds of the cropped image.
        self.crop_x = x;
        self.crop_y = y;
        self.crop_width = width;
        self.crop_height = height;
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
        // Calculate the coordinates within the loaded crop boundaries.
        let crop_x = x.checked_sub(self.crop_x).unwrap();
        let crop_y = y.checked_sub(self.crop_y).unwrap();

        // Check that the pixel location is in bounds. Only need to check width since overruning
        // the height will result in a panic due to out of bounds slice indexing.
        if crop_x >= self.crop_width {
            panic!(
                "access out of loaded image bound: {:?} on {}x{} image with loaded bounds {:?}",
                (x, y),
                self.width,
                self.height,
                (
                    self.crop_x,
                    self.crop_y,
                    self.crop_x + self.crop_width,
                    self.crop_y + self.crop_height
                ),
            );
        }

        <[u8; 3]>::try_from(
            &self.crop_buffer[usize::try_from(crop_y * self.crop_width + crop_x).unwrap()..][..3],
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
        //subimage: oracle.crop_buffer,
        subimage: crop.to_image().into_raw(),
    };
    env::commit(&journal);
}
