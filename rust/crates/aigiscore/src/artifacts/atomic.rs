use std::fs::{self, File, OpenOptions};
use std::io::{self, BufWriter, Write};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

/// Publish a complete file without truncating the previous artifact on failure.
/// This is per-file atomicity, not a transaction over the whole artifact family.
pub(crate) fn write<T>(
    path: &Path,
    render: impl FnOnce(&mut BufWriter<File>) -> io::Result<T>,
) -> io::Result<T> {
    static SEQUENCE: AtomicU64 = AtomicU64::new(0);
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    if parent.join(super::publication::SEAL).try_exists()? {
        return Err(io::Error::new(io::ErrorKind::PermissionDenied, "published artifact generations are immutable"));
    }
    let mut temporary = None;
    for _ in 0..128 {
        let sequence = SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let candidate = parent.join(format!(".aigiscode-{}-{sequence}.tmp", std::process::id()));
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&candidate)
        {
            Ok(file) => {
                temporary = Some((candidate, file));
                break;
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    let (temporary, file) = temporary.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::AlreadyExists,
            "temporary artifact namespace exhausted",
        )
    })?;
    let result = (|| {
        // Preserve an existing regular file's permissions, without following a
        // destination symlink or exposing private content through a wider mode.
        match fs::symlink_metadata(path) {
            Ok(metadata) if metadata.is_file() => file.set_permissions(metadata.permissions())?,
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
        let mut writer = BufWriter::new(file);
        let value = render(&mut writer)?;
        writer.flush()?;
        writer.get_ref().sync_all()?;
        drop(writer);
        fs::rename(&temporary, path)?;
        Ok(value)
    })();
    if let Err(error) = result {
        if let Err(cleanup) = fs::remove_file(&temporary) {
            if cleanup.kind() != io::ErrorKind::NotFound {
                return Err(io::Error::new(
                    error.kind(),
                    format!("{error}; temporary artifact cleanup failed: {cleanup}"),
                ));
            }
        }
        return Err(error);
    }
    result
}
