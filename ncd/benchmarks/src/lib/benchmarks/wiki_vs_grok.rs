use kolmox::{
    compress::{cache::NoCache, Compressor},
    filter::content,
};
use plotly::{
    common::{AxisSide, Marker, Title},
    histogram::Bins, // or plotly::traces::histogram::Bins if needed
    layout::{Axis, BarMode},
    HeatMap,
    Histogram,
    Layout,
    Plot,
};
use rayon::prelude::*;
use tracing::info;

use crate::benchmarks::get_dataset_path;
use crate::dataset;

fn get_content(entry: &dataset::Entry) -> Option<String> {
    let content = std::fs::read_to_string(&entry.filepath).ok()?;

    let txt_content = if entry.page_type == "wiki" {
        content::wiki::get_content(&content)
    } else if entry.page_type == "grok" {
        content::grok::get_content(&content)
    } else {
        None
    };

    let path = entry
        .filepath
        .to_str()
        .unwrap()
        .strip_suffix(".html")
        .unwrap();
    let path = path.to_string() + ".txt";
    std::fs::write(&path, &content).unwrap();
    txt_content
}

pub fn heatmap(
    page_names_wiki: &Vec<String>,
    page_names_grok: &Vec<String>,
    matrix: &Vec<Vec<f64>>,
) -> Plot {
    let heatmap = HeatMap::new(
        page_names_wiki.clone(),
        page_names_grok.clone(),
        matrix.clone(),
    )
    .reverse_scale(true);
    let mut plot = Plot::new();
    plot.add_trace(heatmap);

    let layout = Layout::new()
        .title("NCD Heatmap: Wikipedia vs. Grokpedia".to_string())
        .width(800)
        .height(800)
        .x_axis(
            Axis::new()
                .title(Title::with_text("Wikipedia Article"))
                .side(AxisSide::Bottom)
                .auto_margin(true)
                .tick_angle(-90.0)
                .tick_text(page_names_wiki.clone()),
        )
        .y_axis(
            Axis::new()
                .title("Grokpedia Article")
                .scale_anchor("x")
                .auto_margin(true)
                .tick_text(page_names_grok.clone()),
        );
    plot.set_layout(layout);

    plot
}

pub fn histogram(matrix: &Vec<Vec<f64>>) -> Plot {
    let mut off_values = Vec::new();
    let mut diag_values = Vec::new();

    for i in 0..matrix.len() {
        for j in i..matrix[i].len() {
            let v = matrix[i][j];

            if i == j {
                diag_values.push(v);
            } else {
                off_values.push(v);
            }
        }
    }

    let min = 0.0;
    let max = 1.0;
    let bins = (max - min) / 100.0;

    let hist_off = Histogram::new(off_values)
        .x_bins(Bins::new(min, max, bins))
        .name("Different Subjects")
        .marker(Marker::new().color("lightgray"))
        .opacity(0.6);

    let hist_diag = Histogram::new(diag_values)
        .x_bins(Bins::new(min, max, bins))
        .name("Same Subjects")
        .marker(Marker::new().color("red"))
        .opacity(0.9);

    let mut plot = Plot::new();
    plot.add_trace(hist_off);
    plot.add_trace(hist_diag);

    let layout = Layout::new()
        .title("NCD Score Distribution".to_string())
        .bar_mode(BarMode::Overlay)
        .x_axis(Axis::new().title(Title::with_text("Distance")))
        .y_axis(Axis::new().title("Frequency"));
    plot.set_layout(layout);

    plot
}

pub fn compute_distance_matrix<C: Compressor + Sync>(
    dataset_dir: &str,
    compressor: &C,
) -> (Vec<String>, Vec<String>, Vec<Vec<f64>>) {
    let dataset = dataset::Dataset::new(get_dataset_path(dataset_dir)).unwrap();
    let entries = dataset.entries();
    let entries_wiki = entries
        .into_iter()
        .filter(|e| e.page_type == "wiki")
        .cloned()
        .collect::<Vec<dataset::Entry>>();
    let page_names_wiki = entries_wiki
        .iter()
        .map(|entry| entry.get_name())
        .collect::<Vec<String>>();
    let entries_grok = entries
        .into_iter()
        .filter(|e| e.page_type == "grok")
        .cloned()
        .collect::<Vec<dataset::Entry>>();
    let page_names_grok = entries_grok
        .iter()
        .map(|entry| entry.get_name())
        .collect::<Vec<String>>();

    let matrix = entries_wiki
        .par_iter()
        .map(|entry_wiki| {
            let row = entries_grok
                .iter()
                .map(|entry_grok| {
                    let page_wiki = get_content(entry_wiki).unwrap();
                    let page_grok = get_content(entry_grok).unwrap();
                    compressor.get_distance(&page_wiki, &page_grok)
                })
                .collect::<Vec<f64>>();
            row
        })
        .collect::<Vec<Vec<f64>>>();

    info!("Finished distance matrix computation");

    (page_names_wiki, page_names_grok, matrix)
}

pub fn get_optimal_opts(
    page_wiki_1: &str,
    page_grok_1: &str,
    page_wiki_2: &str,
    page_grok_2: &str,
) -> (u32, u32) {
    let page_wiki_1 = content::wiki::get_content(page_wiki_1).unwrap();
    let page_grok_1 = content::grok::get_content(page_grok_1).unwrap();
    let page_wiki_2 = content::wiki::get_content(page_wiki_2).unwrap();
    let page_grok_2 = content::grok::get_content(page_grok_2).unwrap();

    let mut best_quality = 0;
    let mut best_lg_window_size = 0;
    let mut best_score = f64::MIN;

    for quality in 1..=11 {
        for lg_window_size in 10..=22 {
            let compressor = kolmox::compress::brotli::CompressBrotli::<NoCache>::new(
                quality as u32,
                lg_window_size as u32,
            );

            let dist_w1_g1 = compressor.get_distance(&page_wiki_1, &page_grok_1);
            let dist_w2_g2 = compressor.get_distance(&page_wiki_2, &page_grok_2);
            let dist_w1_w2 = compressor.get_distance(&page_wiki_1, &page_wiki_2);
            let dist_g1_g2 = compressor.get_distance(&page_grok_1, &page_grok_2);
            let dist_w1_g2 = compressor.get_distance(&page_wiki_1, &page_grok_2);
            let dist_g1_w2 = compressor.get_distance(&page_grok_1, &page_wiki_2);

            let score =
                (dist_w1_g2.powi(2) + dist_g1_w2.powi(2) + dist_w1_w2.powi(2) + dist_g1_g2.powi(2))
                    - (dist_w1_g1.powi(2) + dist_w2_g2.powi(2));

            if score > best_score {
                best_score = score;
                best_quality = quality as u32;
                best_lg_window_size = lg_window_size as u32;
            }
        }
    }

    (best_quality, best_lg_window_size)
}

// test
#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod tests {
        use super::*;
        use assert_approx_eq::assert_approx_eq;

        fn read_from_file(file_path: &str) -> String {
            let project_root = env!("CARGO_MANIFEST_DIR");
            let full_path = std::path::Path::new(project_root).join(file_path);
            dbg!(&full_path);
            std::fs::read_to_string(full_path).expect("Failed to read file")
        }

        #[test]
        fn test_compress_grok_and_wiki() {
            let compressor = kolmox::compress::brotli::CompressBrotli::<NoCache>::recommended();
            let page1_g = read_from_file(
                "/Users/anatol/Projects/kolmox/dataset/grokvswiki/page/Web_fiction.html",
            );
            let page1_g = kolmox::filter::content::grok::get_content(&page1_g).unwrap();
            let page1_w = read_from_file(
                "/Users/anatol/Projects/kolmox/dataset/grokvswiki/wiki/Web_fiction.html",
            );
            let page1_w = kolmox::filter::content::wiki::get_content(&page1_w).unwrap();
            let result1 = compressor.get_distance(&page1_g, &page1_w);
            println!("Distance: {}", result1);
            assert_approx_eq!(result1, 0.96, 0.01);
            let page2_g = read_from_file(
                "/Users/anatol/Projects/kolmox/dataset/grokvswiki/page/Arra_San_Agustin.html",
            );
            let page2_g = kolmox::filter::content::grok::get_content(&page2_g).unwrap();
            let result2 = compressor.get_distance(&page2_g, &page1_w);
            println!("Distance: {}", result2);
            assert_approx_eq!(result2, 0.98, 0.01);
        }
    }
}
