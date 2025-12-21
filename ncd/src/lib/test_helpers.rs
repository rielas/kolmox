pub fn read_test_file(path: &str) -> String {
    let project_root = env!("CARGO_MANIFEST_DIR");
    let full_path = std::path::Path::new(project_root).join(path);
    std::fs::read_to_string(full_path).expect("Failed to read test file")
}
