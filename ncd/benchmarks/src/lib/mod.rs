pub mod bench_tests;
pub mod dataset;

use kolmox::{
    compress::{cache::NoCache, Compressor},
    filter::HtmlFilter,
};
use plotly::{
    common::{Marker, Mode, Title},
    layout::{Axis, AxisType},
    ImageFormat, Layout, Plot, Scatter,
};
use std::{path::PathBuf, time::Duration};

pub struct DisplayablePlot {
    plot: Plot,
    width: usize,
    height: usize,
}

impl DisplayablePlot {
    pub fn new(plot: Plot, width: usize, height: usize) -> Self {
        Self { plot, width, height }
    }

    pub fn with_size(mut self, width: usize, height: usize) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn into_inner(self) -> Plot {
        self.plot
    }

    pub fn evcxr_display(&self) {
        self.plot.evcxr_display();

        match self
            .plot
            .to_base64(ImageFormat::PNG, self.width, self.height, 1.0)
        {
            Ok(b64) => {
                println!("EVCXR_BEGIN_CONTENT image/png\n{b64}\nEVCXR_END_CONTENT")
            }
            Err(e) => eprintln!("static image export failed: {e}"),
        }
    }
}

impl From<DisplayablePlot> for Plot {
    fn from(d: DisplayablePlot) -> Self {
        d.plot
    }
}

pub fn read_from_file(file_path: &str) -> String {
    let project_root = env!("CARGO_MANIFEST_DIR");
    let full_path = std::path::Path::new(project_root).join(PathBuf::from(file_path));
    std::fs::read_to_string(full_path).expect("Failed to read file")
}

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

pub fn point_series(results: &[BenchmarkResult]) -> DisplayablePlot {
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

    DisplayablePlot::new(plot, 1000, 700)
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
