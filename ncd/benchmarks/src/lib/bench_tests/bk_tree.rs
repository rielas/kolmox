use bktree::BkTree;
use kolmox::{compress::Compressor, filter::HtmlFilter};
use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Instant,
};
use tracing::{info, warn};

use crate::bench_tests::get_dataset_path;
use crate::dataset;

const DIST_SCALE: f64 = 10_000.0;

fn scale(d: f64) -> isize {
    (d * DIST_SCALE).round() as isize
}

pub fn bk_tree<C: Compressor + 'static>(compressor: C, dataset_name: &str, tolerance: f64) {
    let dataset =
        dataset::Dataset::new(get_dataset_path(dataset_name)).expect("Failed to load dataset");
    let entries = dataset.entries();
    let stripper = kolmox::filter::filter_attributes::FilterHtmlAttributes::default();
    let pages: Arc<Vec<String>> = Arc::new(
        entries
            .iter()
            .map(|e| stripper.process_document(&e.get_content().unwrap()))
            .collect(),
    );
    let n = pages.len();
    let compressor = Arc::new(compressor);
    let tolerance_scaled = scale(tolerance);

    info!(
        dataset = dataset_name,
        pages = n,
        tolerance,
        tolerance_scaled,
        "BK tree benchmark"
    );

    info!("Warming c(x) cache");

    for p in pages.iter() {
        compressor.get_distance(p, p);
    }

    let counter = Arc::new(AtomicUsize::new(0));

    let dist_fn = {
        let pages = Arc::clone(&pages);
        let compressor = Arc::clone(&compressor);
        let counter = Arc::clone(&counter);
        move |a: &usize, b: &usize| -> isize {
            counter.fetch_add(1, Ordering::Relaxed);
            scale(compressor.get_distance(&pages[*a], &pages[*b]))
        }
    };

    counter.store(0, Ordering::Relaxed);
    let build_start = Instant::now();
    let mut tree = BkTree::new(dist_fn);
    tree.insert_all(0..n);
    let build_time = build_start.elapsed();
    let build_calls = counter.load(Ordering::Relaxed);
    info!(?build_time, build_calls, "BK tree built");

    let mut total_brute_calls = 0usize;
    let mut total_tree_calls = 0usize;
    let mut mismatches = 0usize;

    for q in 0..n {
        counter.store(0, Ordering::Relaxed);
        let brute_first: Option<(usize, isize)> = (0..n)
            .map(|i| {
                counter.fetch_add(1, Ordering::Relaxed);
                let d = scale(compressor.get_distance(&pages[i], &pages[q]));
                (i, d)
            })
            .find(|(_, d)| *d <= tolerance_scaled);
        total_brute_calls += counter.load(Ordering::Relaxed);

        counter.store(0, Ordering::Relaxed);
        let tree_results = tree.find(q, tolerance_scaled);
        total_tree_calls += counter.load(Ordering::Relaxed);

        let tree_set: Vec<usize> = tree_results.iter().map(|(i, _)| **i).collect();

        let diverges = match brute_first {
            Some((i, _)) => !tree_set.contains(&i),
            None => !tree_set.is_empty(),
        };
        if diverges {
            mismatches += 1;
            warn!(
                query = q,
                brute_first = ?brute_first.map(|(i, _)| i),
                ?tree_set,
                "BK tree diverges from brute force (NCD triangle inequality violated)"
            );
        }
    }

    info!(
        dataset = dataset_name,
        queries = n,
        mismatches,
        avg_brute_calls = total_brute_calls / n.max(1),
        avg_tree_calls = total_tree_calls / n.max(1),
        "BK tree benchmark complete"
    );
}
