// File-level test conventions shared by input, resolution and detector layers.
pub(crate) fn is_test_source_path(path: &std::path::Path) -> bool {
    let normalized = path.to_string_lossy().replace('\\', "/").to_lowercase();
    if normalized
        .split('/')
        .any(|segment| matches!(segment, "tests" | "test" | "spec" | "__tests__"))
    {
        return true;
    }
    let original_name = match path.file_name().and_then(|n| n.to_str()) {
        Some(name) => name,
        None => return false,
    };
    let file_name = original_name.to_lowercase();
    file_name.ends_with("_test.rs")
        || file_name.ends_with("_test.go")
        || file_name.ends_with("_test.py")
        || file_name.starts_with("test_")
        || original_name.ends_with("Test.php")
        || original_name.ends_with("Spec.php")
        || file_name.ends_with("_test.php")
        || file_name.ends_with("_spec.php")
        || file_name.ends_with("-test.php")
        || file_name.ends_with("-spec.php")
        || matches!(file_name.as_str(), "test.php" | "spec.php")
        || [".test.", ".spec."]
            .iter()
            .any(|marker| file_name.contains(marker))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_conventions_do_not_capture_ordinary_business_names() {
        for path in ["app/Contest.php", "app/Latest.php", "app/Respec.php"] {
            assert!(!super::is_test_source_path(std::path::Path::new(path)));
        }
        for path in [
            "tests/Contest.php",
            "spec/service.rb",
            "app/ServiceTest.php",
            "service_test.php",
            "thing.spec.ts",
        ] {
            assert!(super::is_test_source_path(std::path::Path::new(path)));
        }
    }
}
