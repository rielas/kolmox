//! Python bindings for the `kolmox` Normalized Compression Distance library.
//!
//! Exposes two functions to Python:
//! * [`page_distance`] — NCD between two HTML pages.
//! * [`distance_matrix`] — full pairwise NCD matrix for a list of HTML pages,
//!   computed in parallel with a shared compression cache.

use kolmox::compress::{
    cache::{InMemoryCache, NoCache},
    zstd::CompressZstd,
    Compressor,
};
use kolmox::filter::{filter_attributes::FilterHtmlAttributes, HtmlFilter};
use pyo3::prelude::*;
use rayon::prelude::*;

/// Normalise an HTML page to its structural form: keep tags plus the `id` and
/// `class` attributes, drop all text content. Mirrors what the benchmarks do
/// before measuring distances.
fn strip(html: &str) -> String {
    FilterHtmlAttributes::default().process_document(html)
}

/// Normalized Compression Distance between two HTML pages, in `[0.0, 1.0]`
/// (0 = identical structure, 1 = completely different), using zstd compression.
///
/// `quality` is the zstd level (0–22, default 18 = `recommended()`).
///
/// When `strip_html` is true (default) the pages are reduced to their
/// structural form first, so the distance reflects layout similarity rather
/// than text content.
#[pyfunction]
#[pyo3(signature = (html_a, html_b, quality=18, strip_html=true))]
fn page_distance(html_a: &str, html_b: &str, quality: u32, strip_html: bool) -> f64 {
    let compressor = CompressZstd::<NoCache>::new(quality);
    if strip_html {
        compressor.get_distance(&strip(html_a), &strip(html_b))
    } else {
        compressor.get_distance(html_a, html_b)
    }
}

/// Full symmetric NCD distance matrix for `pages`, using zstd compression.
///
/// Returns an `n x n` list of lists where entry `[i][j]` is the NCD between
/// page `i` and page `j` (diagonal is 0). Rows are computed in parallel and a
/// shared in-memory cache deduplicates per-page and combined compression work,
/// so the cost is close to the `n * (n - 1) / 2` unique pairs.
///
/// `quality` is the zstd level (0–22, default 18 = `recommended()`).
#[pyfunction]
#[pyo3(signature = (pages, quality=18, strip_html=true))]
fn distance_matrix(
    py: Python<'_>,
    pages: Vec<String>,
    quality: u32,
    strip_html: bool,
) -> Vec<Vec<f64>> {
    let stripped: Vec<String> = if strip_html {
        pages.par_iter().map(|p| strip(p)).collect()
    } else {
        pages
    };

    let compressor = CompressZstd::<InMemoryCache>::new(quality);
    let n = stripped.len();

    // Release the GIL while we compress so other Python threads can run.
    py.allow_threads(|| {
        (0..n)
            .into_par_iter()
            .map(|i| {
                (0..n)
                    .map(|j| {
                        if i == j {
                            0.0
                        } else {
                            compressor.get_distance(&stripped[i], &stripped[j])
                        }
                    })
                    .collect()
            })
            .collect()
    })
}

/// The `ncd` Python module.
#[pymodule]
fn ncd(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(page_distance, m)?)?;
    m.add_function(wrap_pyfunction!(distance_matrix, m)?)?;
    Ok(())
}
