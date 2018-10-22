extern crate byteorder;

mod bitstream;
use bitstream::*;

mod huffman;
pub use huffman::*;

mod canonical;
pub use canonical::*;

const NUM_BYTES: usize = 256;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
