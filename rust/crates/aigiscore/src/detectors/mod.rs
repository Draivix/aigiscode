pub mod dead_code;
pub mod hardwiring;

// File-level test conventions shared by the production-code detectors.
fn is_test_source_path(path: &std::path::Path) -> bool {
    let normalized = path.to_string_lossy().replace('\\', "/").to_lowercase();
    if normalized
        .split('/')
        .any(|segment| matches!(segment, "tests" | "test" | "spec" | "__tests__"))
    {
        return true;
    }
    let file_name = match path.file_name().and_then(|n| n.to_str()) {
        Some(name) => name.to_lowercase(),
        None => return false,
    };
    file_name.ends_with("_test.rs")
        || file_name.ends_with("_test.go")
        || file_name.ends_with("_test.py")
        || file_name.starts_with("test_")
        || file_name.ends_with("test.php")
        || file_name.ends_with("spec.php")
        || [".test.", ".spec."]
            .iter()
            .any(|marker| file_name.contains(marker))
}
