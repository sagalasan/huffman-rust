use std::io::{Read, Seek, SeekFrom, Write, BufReader, BufWriter};
use std::fs::File;
use std::path::Path;
use std::error::Error;

use byteorder::{ReadBytesExt, WriteBytesExt, LittleEndian};

use super::*;

/// This struct is used to encode some `Read` using Canonical Huffman codes.
///
/// # Examples
///
/// ```compile_fail
/// use std::io::{BufReader, BufWriter};
/// use std::fs::File;
/// use huffman::Encoder;
///
/// // Open a file
/// let file = File::open(&path)?;
///
/// // Build the encoder
/// let encoder = Encoder::build(BufReader::new(file))?;
///
/// // Create an out file
/// let out_file = File::create(&out_path)?;
///
/// // Encode the file
/// encoder.encode_to(BufWriter::new(out_file))?;
/// ```
pub struct Encoder<R> {
    read: R,
    bytes_read:  u64,
    tree: CanonicalTree,
}

impl<R: Read + Seek> Encoder<R> {
    /// Build the `Encoder` from a `Read`.
    ///
    /// This reads the entire `Read` and then seeks back to the beginning.
    pub fn new(mut read: R) -> Result<Encoder<R>, Box<Error>> {
        // Create a canonical huffman tree
        let (bytes_read, tree) = CanonicalTree::from_read(read.by_ref())?;

        // Reset the read to the beginning
        read.seek(SeekFrom::Start(0))?;

        Ok(Encoder {read, bytes_read, tree})
    }

    /// Encode the encoder to a `Write`
    pub fn encode<W: Write>(&mut self, mut write: W) -> Result<(), Box<Error>> {
        // Write out the size of the original file
        write.write_u64::<LittleEndian>(self.bytes_read)?;

        // Write out the code lengths
        write.write_all(&self.tree.code_lengths())?;

        // Use the tree to encode the read
        self.tree.encode(self.read.by_ref(), write.by_ref())?;

        Ok(())
    }
}

/// This struct is used to decode a file that has been encoded using the `Encoder`
pub struct Decoder<R> {
    read: R,
}

impl<R: Read> Decoder<R> {
    pub fn new(read: R) -> Decoder<R> {
        Decoder { read }
    }

    /// Decode the decoder to a `Read`
    pub fn decode<W: Write>(&mut self, mut write: W) -> Result<(), Box<Error>> {
        // Read the size of the original file
        let bytes: u64 = self.read.read_u64::<LittleEndian>()?;

        // Read in code lengths
        let mut code_buf = [0; 256];
        self.read.read_exact(&mut code_buf)?;

        let code_lengths: Vec<(u8, u8)> = code_buf.iter().enumerate()
            .map(|(i, &l)| (i as u8, l))
            .collect();

        let tree = CanonicalTree::new(code_lengths);

        tree.decode_exact(self.read.by_ref(), write.by_ref(), bytes)?;

        Ok(())
    }
}

/// Helper function to encode files.
pub fn encode_file<P: AsRef<Path>>(in_file: P, out_file: P) -> Result<(), Box<Error>> {
    if out_file.as_ref().exists() {
        return Err(From::from("Out file already exists"));
    }

    let read = BufReader::new(File::open(in_file)?);
    let write = BufWriter::new(File::create(out_file)?);

    let mut encoder = Encoder::new(read)?;

    encoder.encode(write)?;

    Ok(())
}

/// Helper function to decode files.
pub fn decode_file<P: AsRef<Path>>(in_file: P, out_file: P) -> Result<(), Box<Error>> {
    if out_file.as_ref().exists() {
        return Err(From::from("Out file already exists"));
    }

    let read = BufReader::new(File::open(in_file)?);
    let write = BufWriter::new(File::create(out_file)?);

    let mut decoder = Decoder::new(read);

    decoder.decode(write)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::vec::Vec;
    use std::fs::File;

    #[test]
    fn test_small_sample_string() {
        let text = "a small sample string";

        assert!(encode_decode_test(Cursor::new(text)));
    }

    #[test]
    fn test_moby_dick() {
        let file = File::open("./MobyDick.txt").unwrap();

        assert!(encode_decode_test(file));
    }

    #[test]
    fn test_ugly() {
        let file = File::open("./ugly.txt").unwrap();

        assert!(encode_decode_test(file));
    }

    fn encode_decode_test<R: Read + Seek>(mut read: R) -> bool {
        // Read the entire read into memory
        let mut original = Vec::new();
        read.read_to_end(&mut original).unwrap();

        encode_decode_raw_test(&original)
    }

    fn encode_decode_raw_test(bytes: &[u8]) -> bool {
        let original = Cursor::new(bytes);

        let mut encoder = Encoder::new(original).unwrap();

        let mut encoded = Vec::new();
        encoder.encode(&mut encoded).unwrap();

        let mut decoded = Vec::new();
        let mut decoder = Decoder::new(Cursor::new(encoded));
        decoder.decode(&mut decoded).unwrap();

        decoded == bytes
    }
}