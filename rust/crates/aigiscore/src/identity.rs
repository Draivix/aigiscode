use std::path::Path;

pub fn stable_fingerprint(parts: &[&str]) -> String {
    parts
        .iter()
        .map(|part| part.replace('|', "%7C"))
        .collect::<Vec<_>>()
        .join("|")
}

pub fn normalized_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
