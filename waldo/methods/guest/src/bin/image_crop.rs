#![no_main]
// #![no_std]

use divrem::DivCeil;
use image::{GenericImageView, Rgb};
use risc0_zkvm::guest::env;
use waldo_core::merkle::{Node, VectorOracle};
use waldo_core::{Journal, PrivateInput};

risc0_zkvm::guest::entry!(main);

/// ImageOracle provides verified access to an image held by the host.
pub struct ImageOracle<const N: u32> {
    vector: VectorOracle<Vec<u8>>,
    width: u32,
    height: u32,

    crop_x: u32,
    crop_y: u32,
    crop_width: u32,
    crop_height: u32,
    crop_buffer: Vec<u8>,
}

impl<const N: u32> ImageOracle<N> {
    pub fn new(root: Node, width: u32, height: u32) -> Self {
        Self {
            vector: VectorOracle::<Vec<u8>>::new(root),
            width,
            height,
            crop_x: 0,
            crop_y: 0,
            crop_width: 0,
            crop_height: 0,
            crop_buffer: Vec::new(),
        }
    }

    pub fn load_crop(&mut self, x: u32, y: u32, width: u32, height: u32) {
        assert!(self.in_bounds(x + width, y + height));

        // Load the relevant chunks of the image from the host.
        // NOTE: It would be nice to directly serialize into the buffer.
        let width_chunks = DivCeil::div_ceil(self.width, N);
        //let height_chunks = DivCeil::div_ceil(self.height, N);

        // Crop buffer needs to be large enough to hold the chunks to be loaded.
        self.crop_buffer = Vec::with_capacity(
            usize::try_from(DivCeil::div_ceil(width, N) * N * DivCeil::div_ceil(height, N) * N * 3)
                .unwrap(),
        );

        for y_chunk in (y / N)..DivCeil::div_ceil(y + height, N) {
            for x_chunk in (x / N)..DivCeil::div_ceil(x + width, N) {
                self.crop_buffer.extend_from_slice(
                    &self
                        .vector
                        .get(usize::try_from(y_chunk * width_chunks + x_chunk).unwrap()),
                );
                assert!(false);
            }
        }

        // Record the bounds of the cropped image.
        self.crop_x = x;
        self.crop_y = y;
        self.crop_width = width;
        self.crop_height = height;
    }

    pub fn root(&self) -> &Node {
        self.vector.root()
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
        let crop_x = x - self.crop_x;
        let crop_y = y - self.crop_y;

        // TODO: Is it ok to rely upon a subtraction underflow causing a panic?
        // NOTE: I could only check the width and rely on vector bounds checking for height.
        if crop_x >= self.crop_width || crop_y >= self.crop_height {
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
    let mut oracle = ImageOracle::<8>::new(
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

    /*
    let crop = imageops::crop_imm(
        &oracle,
        input.crop_location.0,
        input.crop_location.1,
        input.crop_dimensions.0,
        input.crop_dimensions.1,
    );
    */

    // Collect the verified public information into the journal.
    let journal = Journal {
        root: *oracle.root(),
        image_dimensions: oracle.dimensions(),
        subimage: oracle.crop_buffer, // crop.to_image().into_raw(),
    };
    env::commit(&journal);
}
