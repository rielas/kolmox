use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};

pub trait Cache: Default + Send + Sync {
    fn hash_string(&self, s: &str) -> u64;

    fn get_length_by_hash(&self, hash: u64) -> Option<usize>;

    fn store_length_by_hash(&self, hash: u64, length: usize);
}

#[derive(Clone, Default)]
pub struct NoCache;

impl Cache for NoCache {
    fn hash_string(&self, s: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        hasher.finish()
    }

    fn get_length_by_hash(&self, _hash: u64) -> Option<usize> {
        None
    }

    fn store_length_by_hash(&self, _hash: u64, _length: usize) {}
}

#[derive(Clone)]
pub struct InMemoryCache {
    cache: Arc<RwLock<BTreeMap<u64, usize>>>,
}

impl Default for InMemoryCache {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryCache {
    pub fn new() -> Self {
        InMemoryCache {
            cache: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }

    fn hash_string_internal(s: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        hasher.finish()
    }
}

impl Cache for InMemoryCache {
    fn hash_string(&self, s: &str) -> u64 {
        Self::hash_string_internal(s)
    }

    fn get_length_by_hash(&self, hash: u64) -> Option<usize> {
        let guard = self.cache.read().unwrap();
        guard.get(&hash).cloned()
    }

    fn store_length_by_hash(&self, hash: u64, length: usize) {
        let mut guard = self.cache.write().unwrap();
        guard.insert(hash, length);
    }
}
