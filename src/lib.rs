extern crate byteorder;

use std::io::{Read, Seek, SeekFrom, Write, BufReader, BufWriter};
use std::error::Error;

use byteorder::{ReadBytesExt, WriteBytesExt, LittleEndian};

mod bitstream;
use bitstream::*;

mod huffman;
pub use huffman::*;

mod canonical;
pub use canonical::*;

const NUM_BYTES: usize = 256;

pub fn buffered_encode<R: Read + Seek, W: Write>(read: R, write: W) -> Result<(), Box<Error>> {
    encode(BufReader::new(read), BufWriter::new(write))
}

pub fn buffered_decode<R: Read, W: Write>(read: R, write: W) -> Result<(), Box<Error>> {
    decode(BufReader::new(read), BufWriter::new(write))
}

pub fn encode<R: Read + Seek, W: Write>(mut read: R, mut write: W) -> Result<(), Box<Error>> {
    // Build a canonical huffman tree from read
    let tree = CanonicalTree::from_read(read.by_ref())?;

    // Write the size of the original file
    write.write_u64::<LittleEndian>(tree.bytes)?;

    // Create a buffer for the code lengths
    let mut code_buf = [0; 256];

    // Write out the code lengths
    for i in 0..code_buf.len() {
        code_buf[i] = tree.code_book.get(&(i as u8))
            .map(|v| v.len() as u8)
            .unwrap_or(0);
    }

    write.write(&code_buf)?;

    // Reset the read
    read.seek(SeekFrom::Start(0))?;

    // Encode using canonical tree
    tree.encode(&mut read, &mut write)?;

    Ok(())
}

pub fn decode<R: Read, W: Write>(mut read: R, mut write: W) -> Result<(), Box<Error>> {
    // Read the size of the original file
    let bytes: u64 = read.read_u64::<LittleEndian>()?;

    // Read in code lengths
    let mut code_buf = [0; 256];
    read.read_exact(&mut code_buf)?;

    let code_lengths: Vec<(u8, u8)> = code_buf.iter().enumerate()
        .map(|(i, &l)| (i as u8, l))
        .collect();

    let tree = CanonicalTree::new(bytes, code_lengths);

    tree.decode(&mut read, &mut write)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
