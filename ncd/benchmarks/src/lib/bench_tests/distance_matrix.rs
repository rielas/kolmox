use indicatif::ProgressBar;
use kolmox::{compress::Compressor, filter::HtmlFilter};
use plotly::{
    common::{AxisSide, Title},
    layout::Axis,
    HeatMap, Layout, Plot,
};
use rayon::prelude::*;
use tracing::info;

use crate::bench_tests::get_dataset_path;
use crate::dataset;

type DistanceMatrix = Vec<Vec<f64>>;
type ComputeResult = Result<(Vec<String>, DistanceMatrix), Box<dyn std::error::Error>>;

pub fn heatmap<C: Compressor + Sync>(compressor: &C, dataset_name: &str) -> Plot {
    let (page_names, matrix) = compute_distance_matrix(compressor, dataset_name)
        .expect("Failed to compute distance matrix");

    let heatmap = HeatMap::new(page_names.clone(), page_names.clone(), matrix).reverse_scale(true);

    let mut plot = Plot::new();
    plot.add_trace(heatmap);

    let layout = Layout::new()
        .title(format!(
            "Normalized Compression Distance for {}",
            dataset_name
        ))
        .width(800)
        .height(800)
        .x_axis(
            Axis::new()
                .title(Title::with_text("Page A"))
                .side(AxisSide::Bottom)
                .auto_margin(true)
                .tick_angle(-90.0)
                .tick_text(page_names.clone()),
        )
        .y_axis(
            Axis::new()
                .title("Page B")
                .scale_anchor("x")
                .auto_margin(true)
                .tick_text(page_names.clone()),
        );
    plot.set_layout(layout);

    plot
}

pub fn compute_distance_matrix<C: Compressor + Sync>(
    compressor: &C,
    dataset_name: &str,
) -> ComputeResult {
    let dataset = dataset::Dataset::new(get_dataset_path(dataset_name))?;
    let entries = dataset.entries();
    let page_names = entries
        .iter()
        .map(|entry| entry.get_name())
        .collect::<Vec<String>>();
    info!(
        dataset = dataset_name,
        rows = page_names.len(),
        "Starting distance matrix computation"
    );

    let pb = ProgressBar::new(page_names.len() as u64);
    pb.set_message("computing rows");
    let stripper = kolmox::filter::filter_attributes::FilterHtmlAttributes::default();

    let matrix = entries
        .par_iter()
        .map(|entry_a| {
            let row = entries
                .iter()
                .map(|entry_b| {
                    let page_a = entry_a.get_content().unwrap();
                    let page_b = entry_b.get_content().unwrap();
                    let page_a = stripper.process_document(&page_a);
                    let page_b = stripper.process_document(&page_b);
                    compressor.get_distance(&page_a, &page_b)
                })
                .collect::<Vec<f64>>();
            pb.inc(1);
            row
        })
        .collect::<Vec<Vec<f64>>>();

    pb.finish_and_clear();
    info!(
        dataset = dataset_name,
        "Finished distance matrix computation"
    );

    Ok((page_names, matrix))
}
