use std::path::PathBuf;

pub mod distance_matrix;
pub mod triangle_inequality;
pub mod wiki_vs_grok;

pub fn get_dataset_path(directory: &str) -> PathBuf {
    let project_root = env!("CARGO_MANIFEST_DIR");
    std::path::Path::new(project_root)
        .join("../../dataset")
        .join(directory)
}
