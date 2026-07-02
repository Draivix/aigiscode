//! Fast non-cryptographic content hashing (xxh3) for the change substrate.
//!
//! Used to give every scanned file a stable [`ContentHash`] identity so "unchanged since
//! last run" becomes representable. Not security-sensitive — see [`crate::revision`].

use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

use xxhash_rust::xxh3::{xxh3_64, Xxh3};

use crate::revision::ContentHash;

const HASH_BUFFER_SIZE: usize = 64 * 1024;

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
