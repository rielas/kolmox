use plotly::{
    common::{AxisSide, Title},
    layout::Axis,
    HeatMap, Layout, Plot,
};
use rayon::prelude::*;

use crate::benchmarks::get_dataset_path;
use crate::{benchmarks::Cache, dataset};
use core::brotli_default_filter_attributes;

pub fn heatmap(cache: &mut Cache, dataset_name: &str) {
    let dataset =
        dataset::Dataset::new(get_dataset_path(dataset_name)).expect("Failed to load dataset");
    let entries = dataset.entries();
    let page_names = entries
        .iter()
        .map(|entry| entry.get_name())
        .collect::<Vec<String>>();

    let heatmap = HeatMap::new(
        page_names.clone(),
        page_names.clone(),
        entries
            .par_iter()
            .map(|entry_a| {
                entries
                    .iter()
                    .map(|entry_b| {
                        println!(
                            "Calculating distance between {} and {}",
                            entry_a.url, entry_b.url
                        );
                        cache.calculate(
                            &entry_a.get_content().unwrap(),
                            &entry_b.get_content().unwrap(),
                        )
                    })
                    .collect::<Vec<f64>>()
            })
            .collect::<Vec<Vec<f64>>>(),
    );

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

    plot.show();
}
