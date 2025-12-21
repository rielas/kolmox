use crate::compress::{Cache, Compressor, NoCache};

use std::io::{BufWriter, Write};

pub struct CompressBrotli<C: Cache = NoCache> {
    quality: u32,
    lg_window_size: u32,
    cache: C,
}

impl<C: Cache> CompressBrotli<C> {
    pub fn new(quality: u32, lg_window_size: u32) -> Self {
        Self {
            quality,
            lg_window_size,
            cache: C::default(),
        }
    }

    pub fn recommended() -> Self {
        Self::new(5, 21)
    }

    pub fn max_quality() -> Self {
        Self::new(11, 24)
    }
}

impl<C: Cache> Compressor for CompressBrotli<C> {
    type CacheType = C;

    fn cache(&self) -> &Self::CacheType {
        &self.cache
    }

    fn get_compressed_size(&self, buf: &str) -> usize {
        let mut out = BufWriter::new(Vec::new());
        let buffer_size = buf.len();

        {
            let mut writer = brotli::CompressorWriter::new(
                &mut out,
                buffer_size,
                self.quality,
                self.lg_window_size,
            );
            writer.write_all(buf.as_bytes()).unwrap();
        }

        out.into_inner().unwrap().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::read_test_file as read_from_file;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn test_compress_brotli() {
        let compressor = CompressBrotli::<NoCache>::recommended();
        let page_html =
            read_from_file("../../../dataset/imdb/list/ls541382956/?ref_=tt_urls_2.html");
        let result = compressor.get_distance(&page_html, &page_html);
        assert_approx_eq!(result, 0.0, 0.01);
    }
}
