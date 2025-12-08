use core::brotli_default_filter_attributes;
use std::{
    collections::BTreeMap,
    hash,
    path::PathBuf,
    sync::{Arc, RwLock},
};

pub mod distance_matrix;
pub mod triangle_inequality;

pub struct Cache {
    cache: Arc<RwLock<BTreeMap<(u64, u64), f64>>>,
}

impl Cache {
    pub fn new() -> Self {
        Cache {
            cache: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }

    fn hash_string(s: &str) -> u64 {
        use std::hash::{Hash, Hasher};

        let mut hasher = std::hash::DefaultHasher::new();
        s.hash(&mut hasher);
        hasher.finish()
    }

    pub fn calculate(&self, page_a: &str, page_b: &str) -> f64 {
        let hash_a = Self::hash_string(page_a);
        let hash_b = Self::hash_string(page_b);

        let key = if hash_a <= hash_b {
            (hash_a, hash_b)
        } else {
            (hash_b, hash_a)
        };

        {
            let read_guard = self.cache.read().unwrap();
            if let Some(&cached_result) = read_guard.get(&key) {
                return cached_result;
            }
        }

        let result = brotli_default_filter_attributes(&page_a, &page_b);
        let mut write_guard = self.cache.write().unwrap();
        use std::collections::btree_map::Entry;

        match write_guard.entry(key) {
            Entry::Occupied(o) => *o.get(),
            Entry::Vacant(v) => {
                v.insert(result);
                result
            }
        }
    }
}

pub fn get_dataset_path(directory: &str) -> PathBuf {
    let project_root = env!("CARGO_MANIFEST_DIR");
    std::path::Path::new(project_root)
        .join("../../dataset")
        .join(directory)
}
