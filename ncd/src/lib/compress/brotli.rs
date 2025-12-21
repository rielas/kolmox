use crate::compress::Compressor;

use std::io::{BufWriter, Write};

const QUALITY: u32 = 5;
const LG_WINDOW_SIZE: u32 = 21;

pub struct CompressBrotli {
    quality: u32,
    lg_window_size: u32,
}

impl CompressBrotli {
    pub fn new(quality: u32, lg_window_size: u32) -> Self {
        Self {
            quality,
            lg_window_size,
        }
    }

    pub fn recommended() -> Self {
        Self {
            quality: QUALITY,
            lg_window_size: LG_WINDOW_SIZE,
        }
    }
}

impl Compressor for CompressBrotli {
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
        let compressor = CompressBrotli::recommended();
        let page_html =
            read_from_file("../../../dataset/imdb/list/ls541382956/?ref_=tt_urls_2.html");
        let result = compressor.get_distance(&page_html, &page_html);
        assert_approx_eq!(result, 0.0, 0.01);
    }
}
