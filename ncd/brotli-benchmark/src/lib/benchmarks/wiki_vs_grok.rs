use kolmox::{compress::Compressor, filter::content};
use plotly::{
    common::{AxisSide, Title},
    layout::Axis,
    HeatMap, Layout, Plot,
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

pub fn benchmark(csv_path: &str) -> Plot {
    let dataset_dir = std::path::Path::new(csv_path)
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .ok_or("failed to determine dataset directory from csv path")
        .unwrap();

    let dataset = dataset::Dataset::new(get_dataset_path(dataset_dir)).unwrap();

    let entries = dataset.entries();
    let page_names = entries
        .iter()
        .map(|entry| entry.get_name())
        .collect::<Vec<String>>();
    info!(
        rows = page_names.len(),
        "Starting distance matrix computation"
    );

    let compressor = kolmox::compress::brotli::CompressBrotli::new(11, 24);

    let matrix = entries
        .par_iter()
        .map(|entry_a| {
            let row = entries
                .iter()
                .map(|entry_b| {
                    let page_a = get_content(entry_a).unwrap();
                    let page_b = get_content(entry_b).unwrap();
                    compressor.get_distance(&page_a, &page_b)
                })
                .collect::<Vec<f64>>();
            row
        })
        .collect::<Vec<Vec<f64>>>();
    info!("Finished distance matrix computation");

    let heatmap = HeatMap::new(page_names.clone(), page_names.clone(), matrix);

    let mut plot = Plot::new();
    plot.add_trace(heatmap);

    let layout = Layout::new()
        .title("Normalized Compression Distance for wikipedia vs grokpedia".to_string())
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
            let compressor = kolmox::compress::brotli::CompressBrotli::new(
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
            let compressor = kolmox::compress::brotli::CompressBrotli::recommended();
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
