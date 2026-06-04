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
use crate::bench_tests::vp_tree::VpTree;
use crate::dataset;

const DIST_SCALE: f64 = 10_000.0;

fn scale(d: f64) -> isize {
    (d * DIST_SCALE).round() as isize
}

/// A hand-rolled BK tree with **early-stop** existence search.
///
/// The `bktree` crate only offers a bounded radius search that collects every
/// match, so it can't halt at the first in-range node. We roll our own,
/// stored in an arena (index-based children) to keep the mutable build simple,
/// and driven by the same scaled-integer distance closure as the VP tree.
struct BkNode {
    item: usize,
    /// (edge distance, child node index in the arena).
    children: Vec<(isize, usize)>,
}

struct BkTree<F> {
    nodes: Vec<BkNode>,
    dist: F,
}

impl<F: Fn(&usize, &usize) -> isize> BkTree<F> {
    fn new(dist: F) -> Self {
        Self {
            nodes: Vec::new(),
            dist,
        }
    }

    fn insert_all(&mut self, iter: impl IntoIterator<Item = usize>) {
        for item in iter {
            self.insert(item);
        }
    }

    fn insert(&mut self, item: usize) {
        if self.nodes.is_empty() {
            self.nodes.push(BkNode {
                item,
                children: Vec::new(),
            });
            return;
        }

        let mut idx = 0;
        loop {
            let d = (self.dist)(&self.nodes[idx].item, &item);
            if d == 0 {
                return; // duplicate, nothing to insert
            }
            match self.nodes[idx]
                .children
                .iter()
                .find(|(edge, _)| *edge == d)
                .map(|(_, child)| *child)
            {
                Some(child) => idx = child,
                None => {
                    let new = self.nodes.len();
                    self.nodes.push(BkNode {
                        item,
                        children: Vec::new(),
                    });
                    self.nodes[idx].children.push((d, new));
                    return;
                }
            }
        }
    }

    /// Return the first item found within `radius` of `query`, stopping as soon
    /// as one is located. Returns `None` if nothing is within `radius`.
    fn find_first_within(&self, query: usize, radius: isize) -> Option<(usize, isize)> {
        if self.nodes.is_empty() {
            return None;
        }
        let mut stack = vec![0usize];
        while let Some(idx) = stack.pop() {
            let node = &self.nodes[idx];
            let d = (self.dist)(&query, &node.item);
            if d <= radius {
                return Some((node.item, d)); // early stop
            }
            // Triangle-inequality pruning: only children whose edge distance is
            // within `radius` of `d` can hold a match.
            for (edge, child) in &node.children {
                if (edge - d).abs() <= radius {
                    stack.push(*child);
                }
            }
        }
        None
    }
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
        "BK tree vs VP tree benchmark"
    );

    info!("Warming c(x) cache");

    for p in pages.iter() {
        compressor.get_distance(p, p);
    }

    let counter = Arc::new(AtomicUsize::new(0));

    // A distance closure both trees share, so the number of (expensive) NCD
    // evaluations is measured identically for each structure.
    let make_dist_fn = || {
        let pages = Arc::clone(&pages);
        let compressor = Arc::clone(&compressor);
        let counter = Arc::clone(&counter);
        move |a: &usize, b: &usize| -> isize {
            counter.fetch_add(1, Ordering::Relaxed);
            scale(compressor.get_distance(&pages[*a], &pages[*b]))
        }
    };

    // --- Build BK tree ---
    counter.store(0, Ordering::Relaxed);
    let build_start = Instant::now();
    let mut bk = BkTree::new(make_dist_fn());
    bk.insert_all(0..n);
    let bk_build_time = build_start.elapsed();
    let bk_build_calls = counter.load(Ordering::Relaxed);
    info!(?bk_build_time, bk_build_calls, "BK tree built");

    // --- Build VP tree ---
    counter.store(0, Ordering::Relaxed);
    let build_start = Instant::now();
    let vp = VpTree::new(0..n, make_dist_fn());
    let vp_build_time = build_start.elapsed();
    let vp_build_calls = counter.load(Ordering::Relaxed);
    info!(?vp_build_time, vp_build_calls, "VP tree built");

    let mut total_brute_calls = 0usize;
    let mut total_bk_calls = 0usize;
    let mut total_vp_calls = 0usize;
    let mut bk_mismatches = 0usize;
    let mut vp_mismatches = 0usize;

    for q in 0..n {
        // Brute force: scan until the first page within tolerance (short-circuits).
        counter.store(0, Ordering::Relaxed);
        let brute_has = (0..n)
            .map(|i| {
                counter.fetch_add(1, Ordering::Relaxed);
                scale(compressor.get_distance(&pages[i], &pages[q]))
            })
            .any(|d| d <= tolerance_scaled);
        total_brute_calls += counter.load(Ordering::Relaxed);

        // Both trees stop at the first node within tolerance, just like brute.
        counter.store(0, Ordering::Relaxed);
        let bk_has = bk.find_first_within(q, tolerance_scaled).is_some();
        total_bk_calls += counter.load(Ordering::Relaxed);

        counter.store(0, Ordering::Relaxed);
        let vp_has = vp.find_first_within(q, tolerance_scaled).is_some();
        total_vp_calls += counter.load(Ordering::Relaxed);

        // Each tree must agree with brute force on whether a match exists;
        // disagreement flags an NCD metric anomaly.
        if bk_has != brute_has {
            bk_mismatches += 1;
            warn!(
                query = q,
                brute_has, bk_has, "BK tree disagrees with brute force on match existence"
            );
        }
        if vp_has != brute_has {
            vp_mismatches += 1;
            warn!(
                query = q,
                brute_has, vp_has, "VP tree disagrees with brute force on match existence"
            );
        }
    }

    info!(
        dataset = dataset_name,
        queries = n,
        bk_build_calls,
        vp_build_calls,
        bk_mismatches,
        vp_mismatches,
        avg_brute_calls = total_brute_calls / n.max(1),
        avg_bk_calls = total_bk_calls / n.max(1),
        avg_vp_calls = total_vp_calls / n.max(1),
        "BK tree vs VP tree benchmark complete"
    );
}
