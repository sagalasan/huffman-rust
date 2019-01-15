extern crate byteorder;

mod bitstream;
use bitstream::*;

mod huffman;
pub use huffman::*;

mod canonical;
pub use canonical::*;

mod encode;
pub use encode::*;

const NUM_BYTES: usize = 256;