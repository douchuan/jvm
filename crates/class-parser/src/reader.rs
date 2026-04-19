use std::io::{self, Cursor, Read};
use std::fmt;

#[derive(Debug)]
pub enum Error {
    BadMagic(u32),
    Io(io::Error),
    UnexpectedEof,
    BadConstantPoolTag(u8),
    BadAttributeType(String),
    BadElementTypeTag(u8),
    BadTargetType(u8),
    BadStackMapFrame(u8),
    BadVerificationType(u8),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::BadMagic(m) => write!(f, "bad magic: 0x{:08X}", m),
            Error::Io(e) => write!(f, "io error: {}", e),
            Error::UnexpectedEof => write!(f, "unexpected end of file"),
            Error::BadConstantPoolTag(t) => write!(f, "bad constant pool tag: {}", t),
            Error::BadAttributeType(s) => write!(f, "bad attribute type: {}", s),
            Error::BadElementTypeTag(t) => write!(f, "bad element value tag: 0x{:02X}", t),
            Error::BadTargetType(t) => write!(f, "bad target type: 0x{:02X}", t),
            Error::BadStackMapFrame(t) => write!(f, "bad stack map frame: {}", t),
            Error::BadVerificationType(t) => write!(f, "bad verification type: {}", t),
        }
    }
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

/// Binary reader for class file data.
/// Wraps a Cursor over the input bytes and provides endian-aware read methods.
pub struct Reader {
    cursor: Cursor<Vec<u8>>,
    pos: usize,
}

impl Reader {
    pub fn new(data: Vec<u8>) -> Self {
        let pos = 0;
        Self {
            cursor: Cursor::new(data),
            pos,
        }
    }

    pub fn position(&self) -> usize {
        self.pos
    }

    pub fn read_u8(&mut self) -> Result<u8> {
        let mut buf = [0; 1];
        self.cursor.read_exact(&mut buf).map_err(|e| {
            if e.kind() == io::ErrorKind::UnexpectedEof {
                Error::UnexpectedEof
            } else {
                Error::Io(e)
            }
        })?;
        self.pos += 1;
        Ok(buf[0])
    }

    pub fn read_u16(&mut self) -> Result<u16> {
        let mut buf = [0; 2];
        self.cursor.read_exact(&mut buf).map_err(|e| {
            if e.kind() == io::ErrorKind::UnexpectedEof {
                Error::UnexpectedEof
            } else {
                Error::Io(e)
            }
        })?;
        self.pos += 2;
        Ok(u16::from_be_bytes(buf))
    }

    pub fn read_u32(&mut self) -> Result<u32> {
        let mut buf = [0; 4];
        self.cursor.read_exact(&mut buf).map_err(|e| {
            if e.kind() == io::ErrorKind::UnexpectedEof {
                Error::UnexpectedEof
            } else {
                Error::Io(e)
            }
        })?;
        self.pos += 4;
        Ok(u32::from_be_bytes(buf))
    }

    pub fn read_bytes(&mut self, n: usize) -> Result<Vec<u8>> {
        let mut buf = vec![0; n];
        self.cursor.read_exact(&mut buf).map_err(|e| {
            if e.kind() == io::ErrorKind::UnexpectedEof {
                Error::UnexpectedEof
            } else {
                Error::Io(e)
            }
        })?;
        self.pos += n;
        Ok(buf)
    }

    /// Read a modified UTF-8 string as defined in JVM spec 4.4.7.
    /// Returns the raw bytes (not decoded to Rust String, since JVM uses MUTF-8).
    pub fn read_utf8(&mut self) -> Result<Vec<u8>> {
        let len = self.read_u16()?;
        self.read_bytes(len as usize)
    }
}
