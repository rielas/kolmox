use crate::compress::{
    cache::{Cache, NoCache},
    Compressor,
};
use std::io::Write;

pub struct CompressZstd<C: Cache = NoCache> {
    quality: u32,
    cache: C,
}

impl<C: Cache> CompressZstd<C> {
    pub fn new(quality: u32) -> Self {
        Self {
            quality,
            cache: C::default(),
        }
    }

    pub fn recommended() -> Self {
        Self::new(18)
    }

    pub fn max_quality() -> Self {
        Self::new(22)
    }
}

impl<C: Cache> Compressor for CompressZstd<C> {
    type CacheType = C;

    fn cache(&self) -> &Self::CacheType {
        &self.cache
    }

    fn get_compressed_size(&self, buf: &str) -> usize {
        let mut out = Vec::new();

        {
            let mut encoder = zstd::stream::Encoder::new(&mut out, self.quality as i32).unwrap();
            encoder.write_all(buf.as_bytes()).unwrap();
            encoder.finish().unwrap();
        }

        out.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::read_test_file as read_from_file;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn test_compress_zstd() {
        let compressor = CompressZstd::<NoCache>::recommended();
        let page_html =
            read_from_file("../../../dataset/imdb/list/ls541382956/?ref_=tt_urls_2.html");
        let result = compressor.get_distance(&page_html, &page_html);
        assert_approx_eq!(result, 0.0, 0.01);
    }
}
