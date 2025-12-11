use benchmark::benchmarks::distance_matrix::heatmap;
use benchmark::benchmarks::triangle_inequality;
use benchmark::benchmarks::wiki_vs_grok;
use kolmox::compress::{brotli::CompressBrotli, Compressor};
use std::time::Instant;
use tracing::info;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

fn read_from_file(file_path: &PathBuf) -> String {
    let project_root = env!("CARGO_MANIFEST_DIR");
    let full_path = std::path::Path::new(project_root).join(file_path);
    std::fs::read_to_string(full_path).expect("Failed to read file")
}

fn same_page_with_opts(path: &PathBuf) {
    info!("same-page benchmark (opts)");

    let page_html = read_from_file(path);

    let qualities: Vec<i32> = (3..11).collect();

    for quality in &qualities {
        for lg_window_size in 20..=22 {
            let start = Instant::now();
            let compressor = CompressBrotli::new((*quality) as u32, lg_window_size as u32);
            let result = compressor.get_distance(&page_html, &page_html);
            let duration = start.elapsed();

            info!(
                quality,
                lg_window_size,
                distance = result,
                ?duration,
                "benchmark result"
            );
        }
    }
}

fn main() {
    tracing_subscriber::fmt::init();
    info!("NCD Brotli Benchmark");

    let cli = Cli::parse();
    let cache = &mut benchmark::benchmarks::Cache::new();
    let default_datasets = ["euronews.com", "amazon", "imdb", "wikipedia"];

    match cli.command {
        Commands::SamePage { file } => {
            same_page_with_opts(&file);
        }

        Commands::Heatmap { datasets } => {
            let list: Vec<String> = if datasets.is_empty() {
                default_datasets.iter().map(|s| s.to_string()).collect()
            } else {
                datasets
            };

            for ds in list {
                heatmap(cache, &ds);
            }
        }

        Commands::TriangleInequality { datasets } => {
            let list: Vec<String> = if datasets.is_empty() {
                default_datasets.iter().map(|s| s.to_string()).collect()
            } else {
                datasets
            };

            for ds in list {
                triangle_inequality::triangle_inequality(cache, &ds);
            }
        }

        Commands::WikiVsGrok { csv } => {
            wiki_vs_grok::benchmark(&csv);
        }
        Commands::OptimalOpts {
            wiki1,
            grok1,
            wiki2,
            grok2,
        } => {
            let page_w1 = read_from_file(&wiki1);
            let page_g1 = read_from_file(&grok1);
            let page_w2 = read_from_file(&wiki2);
            let page_g2 = read_from_file(&grok2);

            let (best_q, best_lg) =
                wiki_vs_grok::get_optimal_opts(&page_w1, &page_g1, &page_w2, &page_g2);

            info!(quality = best_q, lg_window_size = best_lg, "optimal opts");
        }
    }
}

/// CLI definitions
#[derive(Parser)]
#[command(name = "brotli-benchmark")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    SamePage {
        file: PathBuf,
    },

    Heatmap {
        #[arg(value_parser)]
        datasets: Vec<String>,
    },

    TriangleInequality {
        #[arg(value_parser)]
        datasets: Vec<String>,
    },

    WikiVsGrok {
        csv: String,
    },

    OptimalOpts {
        /// Wikipedia page (path relative to project root)
        wiki1: PathBuf,
        /// Grok page (path relative to project root)
        grok1: PathBuf,
        /// Second Wikipedia page
        wiki2: PathBuf,
        /// Second Grok page
        grok2: PathBuf,
    },
}
