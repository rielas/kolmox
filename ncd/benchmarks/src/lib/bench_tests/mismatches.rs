use indicatif::ProgressBar;
use kolmox::{compress::Compressor, filter::HtmlFilter};
use rayon::prelude::*;
use std::collections::BTreeMap;
use tracing::{info, warn};

use crate::bench_tests::get_dataset_path;
use crate::dataset;

/// 1-NN classification over NCD: each page is assigned the category of its
/// nearest neighbor; a mismatch is counted when that category differs from
/// the page's own.
pub fn mismatches<C: Compressor + Sync>(compressor: &C, dataset_name: &str) {
    let dataset =
        dataset::Dataset::new(get_dataset_path(dataset_name)).expect("Failed to load dataset");
    let entries = dataset.entries();
    let stripper = kolmox::filter::filter_attributes::FilterHtmlAttributes::default();
    let pages: Vec<String> = entries
        .iter()
        .map(|e| stripper.process_document(&e.get_content().unwrap()))
        .collect();
    let n = pages.len();

    info!(
        dataset = dataset_name,
        pages = n,
        "1-NN category mismatch benchmark"
    );

    let pb = ProgressBar::new(n as u64);
    pb.set_message("finding nearest neighbors");

    let nearest: Vec<Option<usize>> = (0..n)
        .into_par_iter()
        .map(|i| {
            let neighbor = (0..n)
                .filter(|&j| j != i)
                .map(|j| (j, compressor.get_distance(&pages[i], &pages[j])))
                .min_by(|(_, a), (_, b)| a.total_cmp(b))
                .map(|(j, _)| j);
            pb.inc(1);
            neighbor
        })
        .collect();

    pb.finish_and_clear();

    let mut page_mismatches = 0usize;
    // category -> (mismatched, total)
    let mut per_category: BTreeMap<&str, (usize, usize)> = BTreeMap::new();

    for (i, neighbor) in nearest.iter().enumerate() {
        let actual = entries[i].page_type.as_str();
        let stats = per_category.entry(actual).or_default();
        stats.1 += 1;

        let Some(j) = neighbor else { continue };
        let predicted = entries[*j].page_type.as_str();

        if predicted != actual {
            page_mismatches += 1;
            stats.0 += 1;
            warn!(
                page = entries[i].url,
                actual,
                predicted,
                neighbor = entries[*j].url,
                "category mismatch"
            );
        }
    }

    for (category, (mismatched, total)) in &per_category {
        info!(
            dataset = dataset_name,
            category, mismatched, total, "category result"
        );
    }

    let categories_with_mismatches = per_category.values().filter(|(m, _)| *m > 0).count();

    info!(
        dataset = dataset_name,
        pages = n,
        page_mismatches,
        categories = per_category.len(),
        categories_with_mismatches,
        "mismatch benchmark complete"
    );
}
