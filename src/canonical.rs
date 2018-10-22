use std::io::{Read, Write};
use std::collections::{Bound, HashMap, BTreeMap};
use std::result::Result;
use std::error::Error;

use super::*;

const MAX_U64_MASK: u64 = 1 << 63;

pub type CodeBook = HashMap<u8, Vec<bool>>;

#[derive(Debug)]
pub struct LookupEntry {
    length: u8,
    codes: Vec<u8>,
}

impl LookupEntry {
    pub fn new(length: u8, codes: Vec<u8>) -> LookupEntry {
        LookupEntry {length, codes}
    }
}

pub struct CanonicalTree {
    pub bytes: u64,
    pub code_book: CodeBook,
    lookup: BTreeMap<u64, LookupEntry>,
}

impl CanonicalTree {
    pub fn new(bytes: u64, code_lengths: Vec<(u8, u8)>) -> CanonicalTree {
        // Build the canonical code book
        let code_book = canonical_code_book(&code_lengths);

        // Build the lookup tree
        let lookup = lookup_tree(&code_book);

        CanonicalTree {
            bytes,
            code_book,
            lookup,
        }
    }

    pub fn from_read<R: Read>(read: R) -> Result<CanonicalTree, Box<Error>> {
        // Keep track of state
        let mut bytes_read: u64 = 0;
        let mut freq_table: [u64; NUM_BYTES] = [0; NUM_BYTES];

        for byte in read.bytes() {
            if bytes_read == u64::max_value() {
                return Err(From::from(format!("Cannot read file larger than {} bytes", u64::max_value())));
            }
            bytes_read += 1;
            freq_table[byte? as usize] += 1;
        }

        // Read was empty
        if bytes_read == 0 {
            return Err(From::from("Read was empty"));
        }

        // Create a huffman from the frequencies
        let huff_tree = HuffmanTree::new(&freq_table)
            .ok_or("Could not create buffman tree")?;

        // Get code lengths from huffman tree
        let code_lengths = huff_tree.get_code_lengths();

        Ok(CanonicalTree::new(bytes_read, code_lengths))
    }

    pub fn encode<R: Read, W: Write>(&self, read: & mut R, write: & mut W) -> Result<(), Box<Error>> {
        let mut bit_writer = BitWriter::new(write);

        for byte_res in read.bytes() {
            let byte = byte_res?;
            let code = self.code_book.get(&byte)
                .ok_or(format!("Symbol {} not found in code book", byte))?;

            bit_writer.write_bits(&code)?;
        }

        Ok(())
    }

    pub fn decode<R: Read, W: Write>(&self, read: & mut R, write: & mut W) -> Result<(), Box<Error>> {
        let mut bit_reader = BitReader::new(read);

        let mut buf: [u8; 1] = [0; 1];
        let mut code: u64 = 0;
        let mut mask: u64 = MAX_U64_MASK;
        let mut offset: u64 = 0;

        loop {
            if let Some(bit) = bit_reader.read_bit()? {
                if bit {
                    code |= mask;
                }

                mask >>= 1;
                offset += 1;

                if mask > 0 {
                    continue;
                }
            } else if offset == 0 {
                return Ok(())
            }

            // Find the lookup entry
            let (&min_code, entry) = self.lookup.range((Bound::Unbounded, Bound::Included(code)))
                .next_back()
                .ok_or("File corrupt")?;

            // Index into the entry
            let index = (code - min_code) >> (64 - entry.length);

            // Lookup the index in the entry
            buf[0] = entry.codes[index as usize];

            // Write out the byte
            write.write(&buf)?;

            // Clear the first entry.length bits and left shift the code
            mask = MAX_U64_MASK;
            for _ in 0..entry.length {
                code &= !mask;
                mask >>= 1;
            }

            code <<= entry.length;
            offset -= entry.length as u64;
            mask = 1 << entry.length as u64 - 1;
        }
    }
}

pub fn canonical_code_book(code_lengths: &[(u8, u8)]) -> CodeBook {
    // Sort by code_length and then by symbol
    let mut sorted = Vec::from(code_lengths);
    sorted.sort_by_key(|&(symbol, length)| (length,  symbol));

    let mut result = HashMap::new();

    // Current code
    let mut code: u64 = 0;

    let mut iter = sorted.iter().peekable();
    while let Some(&(symbol, length)) = iter.next() {
        result.insert(symbol, code_to_vec(length, code));

        if let Some(&&(_symbol_next, length_next)) = iter.peek() {
            code = (code + 1) << (length_next - length);
        }
    }

    result
}

#[inline]
fn code_to_vec(length: u8, code: u64) -> Vec<bool> {
    let mut vec = Vec::with_capacity(length as usize);
    let mut mask = 1 << ((length - 1) as u64);

    for _ in 0..(length as u64) {
        vec.push((mask & code) != 0);
        mask >>= 1;
    }

    vec
}

pub fn lookup_tree(code_book: &CodeBook) -> BTreeMap<u64, LookupEntry> {
    let mut tree = BTreeMap::new();

    // Group by lengths
    let mut map: HashMap<usize, Vec<(u8, u64)>> = HashMap::new();

    for (&symbol, code_vec) in code_book.iter() {
        let vec = map.entry(code_vec.len())
            .or_insert(Vec::new());

        let mut mask: u64 = MAX_U64_MASK;
        let mut code: u64 = 0;

        for &bit in code_vec.iter() {
            if bit {
                code |= mask;
            }

            mask >>= 1;
        }

        vec.push((symbol, code));
    }

    // Create the entries to put into the tree
    for (&length, &ref vec) in map.iter() {
        let min_code = vec.iter()
            .map(|&(_symbol, code)| code)
            .min()
            .expect(&format!("No codes for length {}", length));

        let mut symbols: Vec<u8> = vec.iter()
            .map(|&(symbol, _code)| symbol)
            .collect();
        symbols.sort();

        let entry = LookupEntry::new(length as u8, symbols);

        tree.insert(min_code, entry);
    }

    tree
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::vec::Vec;

    #[test]
    fn test_small_sample_string() {
        let text = "a small sample string";

        assert!(encode_decode_test(text.as_bytes()));
    }

    fn encode_decode_test(text: &[u8]) -> bool {
        let mut encoded_cursor = Cursor::new(text);
        let tree = CanonicalTree::from_read(&mut encoded_cursor).unwrap();
        encoded_cursor = Cursor::new(text);

        let mut encoded = Vec::new();

        tree.encode(&mut encoded_cursor, &mut encoded).unwrap();

        let mut decoded = Vec::new();

        tree.decode(&mut Cursor::new(encoded), &mut decoded).unwrap();

        decoded == text
    }
}