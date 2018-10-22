use std::io;
use std::io::{Read, Write};

const MAX_MASK: u8 = 1 << 7;

pub struct BitReader<T> {
    read: T,
    buf: [u8; 1],
    current: u8,
    mask: u8
}

impl <T: Read> BitReader<T> {
    pub fn new(read: T) -> BitReader<T> {
        BitReader {
            read,
            buf: [0; 1],
            current: 0,
            mask: 0,
        }
    }

    pub fn read_bit(&mut self) -> io::Result<Option<bool>> {
        if self.mask == 0 {
            match self.read_next_byte()? {
                None => return Ok(None),
                _ => (),
            }
        }

        let bit = (self.current & self.mask) != 0;
        self.mask >>= 1;

        Ok(Some(bit))
    }

    fn read_next_byte(&mut self) -> io::Result<Option<()>> {
        let bytes_read = self.read.read(&mut self.buf)?;

        if bytes_read == 0 {
            return Ok(None);
        }

        self.current = self.buf[0];
        self.mask = MAX_MASK;

        Ok(Some(()))
    }
}

pub struct BitWriter<T: Write> {
    write: T,
    buf: [u8; 1],
    current: u8,
    mask: u8,
}

impl <T: Write> BitWriter<T> {

    pub fn new(write: T) -> BitWriter<T> {
        BitWriter {
            write,
            buf: [0; 1],
            current: 0,
            mask: MAX_MASK,
        }
    }

    pub fn write_bit(&mut self, bit: bool) -> io::Result<()> {
        if self.mask == 0 {
            self.write_current_byte()?;
        }

        if bit {
            self.current |= self.mask;
        } else {
            self.current &= !self.mask;
        }

        self.mask >>= 1;

        Ok(())
    }

    pub fn write_bits(&mut self, bits: &[bool]) -> io::Result<()> {
        for &bit in bits.iter() {
            self.write_bit(bit)?;
        }

        Ok(())
    }

    fn write_current_byte(&mut self) -> io::Result<()> {
        self.buf[0] = self.current;

        self.write.write(&self.buf)?;

        self.current = 0;
        self.mask = MAX_MASK;

        Ok(())
    }
}

impl<T: Write> Drop for BitWriter<T> {
    fn drop(&mut self) {
        if self.mask != MAX_MASK {
            let _ = self.write_current_byte();
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::vec::Vec;

    #[test]
    fn test_empty_reader() {
        let mut bit_reader = BitReader::new(Cursor::new(Vec::new()));

        assert!(bit_reader.read_bit().unwrap().is_none());
        assert!(bit_reader.read_bit().unwrap().is_none());
    }

    #[test]
    fn test_reader() {
        let mut bit_reader = BitReader::new( Cursor::new(vec![243, 98]));

        // 11110011 01100010
        assert_eq!(bit_reader.read_bit().unwrap().unwrap(), true);
        assert_eq!(bit_reader.read_bit().unwrap().unwrap(), true);
        assert_eq!(bit_reader.read_bit().unwrap().unwrap(), true);
        assert_eq!(bit_reader.read_bit().unwrap().unwrap(), true);
        assert_eq!(bit_reader.read_bit().unwrap().unwrap(), false);
        assert_eq!(bit_reader.read_bit().unwrap().unwrap(), false);
        assert_eq!(bit_reader.read_bit().unwrap().unwrap(), true);
        assert_eq!(bit_reader.read_bit().unwrap().unwrap(), true);

        assert_eq!(bit_reader.read_bit().unwrap().unwrap(), false);
        assert_eq!(bit_reader.read_bit().unwrap().unwrap(), true);
        assert_eq!(bit_reader.read_bit().unwrap().unwrap(), true);
        assert_eq!(bit_reader.read_bit().unwrap().unwrap(), false);
        assert_eq!(bit_reader.read_bit().unwrap().unwrap(), false);
        assert_eq!(bit_reader.read_bit().unwrap().unwrap(), false);
        assert_eq!(bit_reader.read_bit().unwrap().unwrap(), true);
        assert_eq!(bit_reader.read_bit().unwrap().unwrap(), false);

        assert!(bit_reader.read_bit().unwrap().is_none());
    }

    #[test]
    fn test_writer() {
        let mut vec: Vec<u8> = Vec::new();
        {
            let mut bit_writer = BitWriter::new(vec.by_ref());

            // 11110011 01100010
            assert!(bit_writer.write_bit(true).is_ok());
            assert!(bit_writer.write_bit(true).is_ok());
            assert!(bit_writer.write_bit(true).is_ok());
            assert!(bit_writer.write_bit(true).is_ok());
            assert!(bit_writer.write_bit(false).is_ok());
            assert!(bit_writer.write_bit(false).is_ok());
            assert!(bit_writer.write_bit(true).is_ok());
            assert!(bit_writer.write_bit(true).is_ok());

            assert!(bit_writer.write_bit(false).is_ok());
            assert!(bit_writer.write_bit(true).is_ok());
            assert!(bit_writer.write_bit(true).is_ok());
            assert!(bit_writer.write_bit(false).is_ok());
            assert!(bit_writer.write_bit(false).is_ok());
            assert!(bit_writer.write_bit(false).is_ok());
            assert!(bit_writer.write_bit(true).is_ok());
            assert!(bit_writer.write_bit(false).is_ok());
        }

        assert_eq!(vec.len(), 2);
        assert_eq!(vec[0], 243);
        assert_eq!(vec[1], 98);
    }

    #[test]
    fn test_writer_partial() {
        let mut vec: Vec<u8> = Vec::new();
        {
            let mut bit_writer = BitWriter::new(vec.by_ref());

            // 11110011 01100
            assert!(bit_writer.write_bit(true).is_ok());
            assert!(bit_writer.write_bit(true).is_ok());
            assert!(bit_writer.write_bit(true).is_ok());
            assert!(bit_writer.write_bit(true).is_ok());
            assert!(bit_writer.write_bit(false).is_ok());
            assert!(bit_writer.write_bit(false).is_ok());
            assert!(bit_writer.write_bit(true).is_ok());
            assert!(bit_writer.write_bit(true).is_ok());

            assert!(bit_writer.write_bit(false).is_ok());
            assert!(bit_writer.write_bit(true).is_ok());
            assert!(bit_writer.write_bit(true).is_ok());
            assert!(bit_writer.write_bit(false).is_ok());
            assert!(bit_writer.write_bit(false).is_ok());
        }

        assert_eq!(vec.len(), 2);
        assert_eq!(vec[0], 243);
        assert_eq!(vec[1], 96);
    }

    #[test]
    fn test_drop_no_panic() {
        struct FailOnFlush {}
        impl Write for FailOnFlush {
            fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
                Ok(buf.len())
            }
            fn flush(&mut self) -> io::Result<()> {
                Err(io::Error::last_os_error())
            }
        }

        {
            let mut bit_writer = BitWriter::new(FailOnFlush{});

            assert!(bit_writer.write_bit(true).is_ok());
            assert!(bit_writer.write_bit(true).is_ok());
            assert!(bit_writer.write_bit(false).is_ok());
        }
    }
}

