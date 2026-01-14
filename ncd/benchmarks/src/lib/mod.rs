pub mod benchmarks;
pub mod dataset;

use kolmox::{
    compress::{cache::NoCache, Compressor},
    filter::HtmlFilter,
};
use plotly::{
    common::{Marker, Mode, Title},
    layout::{Axis, AxisType},
    Layout, Plot, Scatter,
};
use std::time::Duration;

pub fn brotli_filter_attributes(page_a: &str, page_b: &str) -> f64 {
    let stripper = kolmox::filter::filter_attributes::FilterHtmlAttributes::default();
    let stripped_a = stripper.process_document(page_a);
    let stripped_b = stripper.process_document(page_b);
    let compressor = kolmox::compress::brotli::CompressBrotli::<NoCache>::recommended();
    compressor.get_distance(&stripped_a, &stripped_b)
}

#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub quality: u32,
    pub lg_window_size: u32,
    pub compression_ratio: f64,
    pub duration: Duration,
}

pub fn point_series(results: &[BenchmarkResult]) -> Plot {
    let mut x0: Vec<f64> = Vec::new();
    let mut y0: Vec<f64> = Vec::new();
    let mut t0: Vec<String> = Vec::new();

    let mut x1: Vec<f64> = Vec::new();
    let mut y1: Vec<f64> = Vec::new();
    let mut t1: Vec<String> = Vec::new();

    let mut x2: Vec<f64> = Vec::new();
    let mut y2: Vec<f64> = Vec::new();
    let mut t2: Vec<String> = Vec::new();

    let mut x3: Vec<f64> = Vec::new();
    let mut y3: Vec<f64> = Vec::new();
    let mut t3: Vec<String> = Vec::new();

    for r in results {
        let xs = r.duration.as_secs_f64();
        let ys = r.compression_ratio;
        let txt = format!("lg_window: {}, quality: {}", r.lg_window_size, r.quality);

        match r.quality {
            0..=2 => {
                x0.push(xs);
                y0.push(ys);
                t0.push(txt);
            }
            3..=5 => {
                x1.push(xs);
                y1.push(ys);
                t1.push(txt);
            }
            6..=8 => {
                x2.push(xs);
                y2.push(ys);
                t2.push(txt);
            }
            _ => {
                x3.push(xs);
                y3.push(ys);
                t3.push(txt);
            }
        }
    }

    let mut plot = Plot::new();

    if !x0.is_empty() {
        let trace = Scatter::new(x0, y0)
            .mode(Mode::Markers)
            .hover_text_array(t0)
            .name("Q 0-2")
            .marker(Marker::new().color("red").size(8));
        plot.add_trace(trace);
    }

    if !x1.is_empty() {
        let trace = Scatter::new(x1, y1)
            .mode(Mode::Markers)
            .hover_text_array(t1)
            .name("Q 3-5")
            .marker(Marker::new().color("blue").size(8));
        plot.add_trace(trace);
    }

    if !x2.is_empty() {
        let trace = Scatter::new(x2, y2)
            .mode(Mode::Markers)
            .hover_text_array(t2)
            .name("Q 6-8")
            .marker(Marker::new().color("green").size(8));
        plot.add_trace(trace);
    }

    if !x3.is_empty() {
        let trace = Scatter::new(x3, y3)
            .mode(Mode::Markers)
            .hover_text_array(t3)
            .name("Q 9+")
            .marker(Marker::new().color("magenta").size(8));
        plot.add_trace(trace);
    }

    let layout = Layout::new()
        .title("NCD Brotli: distance and compression time")
        .x_axis(Axis::new().type_(AxisType::Log).title(Title::with_text(
            "Compression Time (seconds). The lower, the better",
        )))
        .y_axis(
            Axis::new()
                .type_(AxisType::Log)
                .title(Title::with_text("Distance. The lower, the better")),
        );

    plot.set_layout(layout);

    plot
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn test_simple_pages() {
        let page_a = r#"<html>
    <head>
        <title>This title</title>
    </head>
    <body>
        <p class="hello">Hello, world!</p>
    </body>
</html>"#;
        let page_b = r#"<html>
    <head>
        <title>A Different Test</title>
    </head>
    <body>
        <p class="hello">Good bye, world!</p>
    </body>
</html>"#;
        assert_approx_eq!(brotli_filter_attributes(page_a, page_b), 0.0, 0.1);
    }
}
