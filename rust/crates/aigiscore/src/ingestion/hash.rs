//! Fast non-cryptographic content hashing (xxh3) for the change substrate.
//!
//! Used to give every scanned file a stable [`ContentHash`] identity so "unchanged since
//! last run" becomes representable. Not security-sensitive — see [`crate::revision`].

use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;

use xxhash_rust::xxh3::{xxh3_64, Xxh3};

use crate::revision::ContentHash;

const HASH_BUFFER_SIZE: usize = 64 * 1024;

/// Hash exactly the bytes transferred, including partial reads/writes.
pub struct HashingIo<T> {
    inner: T,
    hasher: Xxh3,
}

impl<T> HashingIo<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            hasher: Xxh3::new(),
        }
    }
    pub fn content_hash(&self) -> ContentHash {
        ContentHash(self.hasher.digest())
    }
}

impl<T: Read> Read for HashingIo<T> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        let count = self.inner.read(buffer)?;
        self.hasher.update(&buffer[..count]);
        Ok(count)
    }
}

impl<T: Write> Write for HashingIo<T> {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        let count = self.inner.write(buffer)?;
        self.hasher.update(&buffer[..count]);
        Ok(count)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

/// Stream a file through xxh3-64 without holding the whole file in memory.
pub fn hash_file_xxh3(path: &Path) -> io::Result<ContentHash> {
    let mut file = File::open(path)?;
    let mut hasher = Xxh3::new();
    let mut buf = [0_u8; HASH_BUFFER_SIZE];
    loop {
        let read = file.read(&mut buf)?;
        if read == 0 {
            break;
        }
        hasher.update(&buf[..read]);
    }
    Ok(ContentHash(hasher.digest()))
}

/// Hash an in-memory byte slice (e.g. content already read for parsing).
#[must_use]
pub fn hash_bytes_xxh3(bytes: &[u8]) -> ContentHash {
    ContentHash(xxh3_64(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_file(name: &str, contents: &[u8]) -> std::path::PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("aigiscore-hash-{nonce}"));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join(name);
        fs::write(&path, contents).unwrap();
        path
    }

    #[test]
    fn streaming_hashes_match_transferred_bytes() {
        let mut writer = HashingIo::new(Vec::new());
        writer.write_all(b"first").unwrap();
        writer.write_all(b"second").unwrap();
        assert_eq!(writer.content_hash(), hash_bytes_xxh3(b"firstsecond"));
        let mut reader = HashingIo::new(std::io::Cursor::new(b"firstsecond"));
        let mut output = Vec::new();
        reader.read_to_end(&mut output).unwrap();
        assert_eq!(output, b"firstsecond");
        assert_eq!(reader.content_hash(), writer.content_hash());
    }

    #[test]
    fn file_and_bytes_hashes_agree_and_are_content_sensitive() {
        let path = temp_file("a.txt", b"hello world");
        assert_eq!(
            hash_file_xxh3(&path).unwrap(),
            hash_bytes_xxh3(b"hello world")
        );
        assert_ne!(
            hash_bytes_xxh3(b"hello world"),
            hash_bytes_xxh3(b"hello_world")
        );
    }

    #[test]
    fn empty_file_hashes_without_error() {
        let path = temp_file("empty.txt", b"");
        assert_eq!(hash_file_xxh3(&path).unwrap(), hash_bytes_xxh3(b""));
    }
}
