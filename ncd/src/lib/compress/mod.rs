pub mod brotli;
pub mod cache;

use cache::Cache;

use std::cmp;

pub trait Compressor {
    type CacheType: Cache;

    fn cache(&self) -> &Self::CacheType;

    fn get_distance(&self, page_a: &str, page_b: &str) -> f64 {
        let length_combined = self.get_combined_length(page_a, page_b);
        let a_compressed = self.get_compressed_size(page_a);
        let b_compressed = self.get_compressed_size(page_b);

        let min = cmp::min(a_compressed, b_compressed);
        let max = cmp::max(a_compressed, b_compressed);

        if length_combined < min {
            return 0.0;
        }

        (length_combined - min) as f64 / max as f64
    }

    fn get_compressed_size(&self, buf: &str) -> usize;

    fn get_combined_length(&self, page_a: &str, page_b: &str) -> usize {
        let page_ab = page_a.to_owned() + page_b;
        let hash_ab = self.cache().hash_string(&page_ab);
        let page_ba = page_b.to_owned() + page_a;
        let hash_ba = self.cache().hash_string(&page_ba);

        if let Some(cached) = self.cache().get_length_by_hash(hash_ab) {
            cached
        } else if let Some(cached) = self.cache().get_length_by_hash(hash_ba) {
            cached
        } else {
            let length_combined_a_b = self.get_compressed_size(&page_ab);
            let length_combined_b_a = self.get_compressed_size(&page_ba);
            let res = cmp::min(length_combined_a_b, length_combined_b_a);
            self.cache().store_length_by_hash(hash_ab, res);
            self.cache().store_length_by_hash(hash_ba, res);
            res
        }
    }
}
