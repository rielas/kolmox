use benchmark::benchmarks::distance_matrix::heatmap;
use benchmark::benchmarks::triangle_inequality;
use benchmark::dataset;
use core::brotli_default_filter_attributes;
use core::compress::{brotli::CompressBrotli, Compressor};
use plotly::common::{AxisSide, Title};
use plotly::layout::Axis;
use plotly::{HeatMap, Layout, Plot};
use rayon::prelude::*;
use std::path::PathBuf;
use std::time::{Duration, Instant};

fn read_from_file(file_path: &str) -> String {
    let project_root = env!("CARGO_MANIFEST_DIR");
    let full_path = std::path::Path::new(project_root).join(file_path);
    std::fs::read_to_string(full_path).expect("Failed to read file")
}

const FILE_PATH: &str = "../../dataset/imdb/list/ls541382956/?ref_=tt_urls_2.html";

#[derive(Debug, Clone)]
struct BenchmarkResult {
    quality: u32,
    lg_window_size: u32,
    compression_ratio: f64,
    duration: Duration,
}

fn same_page() {
    println!("A distance betwwen the same pages should be close to 0.0");
    let page_html = read_from_file(FILE_PATH);
    let mut results = Vec::new();

    for quality in 3..11 {
        for lg_window_size in 20..=22 {
            let start = Instant::now();
            let compressor = CompressBrotli::new(quality, lg_window_size);
            let result = compressor.get_distance(&page_html, &page_html);
            let duration = start.elapsed();

            let benchmark_result = BenchmarkResult {
                quality,
                lg_window_size,
                compression_ratio: result,
                duration,
            };

            results.push(benchmark_result.clone());

            println!(
                "Quality: {quality}, LG Window Size: {lg_window_size}, Distance: {:.6}, Time: {:?}",
                result, duration
            );
        }
    }
}

fn main() {
    println!("NCD Brotli Benchmark");
    same_page();

    let cache = &mut benchmark::benchmarks::Cache::new();

    for dataset in ["euronews.com", "amazon", "imdb", "wikipedia"] {
        heatmap(cache, dataset);
        triangle_inequality::triangle_inequality(cache, dataset);
    }
}
