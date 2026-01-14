use kolmox::{
    compress::{cache::NoCache, Compressor},
    filter::content,
};
use plotly::{
    common::{AxisSide, DashType, Line, Marker, Mode, Title},
    histogram::Bins,
    layout::{Axis, BarMode},
    HeatMap,
    Histogram,
    Layout,
    Plot,
    Scatter,
};
use rayon::prelude::*;
use statrs::statistics::{Data, OrderStatistics};
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
        .title("Normalized Compression Distance for wikipedia vs grokpedia".to_string())
        .width(800)
        .height(800)
        .x_axis(
            Axis::new()
                .title(Title::with_text("Wikipedia Pages"))
                .side(AxisSide::Bottom)
                .auto_margin(true)
                .tick_angle(-90.0)
                .tick_text(page_names_wiki.clone()),
        )
        .y_axis(
            Axis::new()
                .title("Grokpedia Pages")
                .scale_anchor("x")
                .auto_margin(true)
                .tick_text(page_names_grok.clone()),
        );
    plot.set_layout(layout);

    plot
}

pub fn histogram(matrix: &Vec<Vec<f64>>) -> Plot {
    let mut related_pages = Vec::new();
    let mut nonrelated_pages = Vec::new();

    for i in 0..matrix.len() {
        for j in i..matrix[i].len() {
            let v = matrix[i][j];

            if i == j {
                related_pages.push(v);
            } else {
                nonrelated_pages.push(v);
            }
        }
    }

    let min = 0.0;
    let max = 1.0;
    let nbins = 100usize;
    let bin_size = (max - min) / nbins as f64;

    let hist_nonrelated = Histogram::new(nonrelated_pages.clone())
        .x_bins(Bins::new(min, max, bin_size))
        .name("Nonrelated Subject Pages")
        .marker(Marker::new().color("lightgray"))
        .opacity(0.6);

    let hist_related = Histogram::new(related_pages.clone())
        .x_bins(Bins::new(min, max, bin_size))
        .name("Same Subject Pages")
        .marker(Marker::new().color("red"))
        .opacity(0.9);

    let mut plot = Plot::new();
    plot.add_trace(hist_nonrelated);
    plot.add_trace(hist_related);

    let med_nonrelated = if nonrelated_pages.is_empty() {
        0.0
    } else {
        let mut tmp = nonrelated_pages.clone();
        Data::new(&mut tmp).median()
    };

    let med_related = if related_pages.is_empty() {
        0.0
    } else {
        let mut tmp = related_pages.clone();
        Data::new(&mut tmp).median()
    };

    const LINE_HEIGHT: f64 = 100.0;

    let vline_nonrelated =
        Scatter::new(vec![med_nonrelated, med_nonrelated], vec![0.0, LINE_HEIGHT])
            .mode(Mode::Lines)
            .name("Nonrelated Subject Pages median")
            .line(Line::new().color("black").width(1.0).dash(DashType::Dash))
            .opacity(0.6);

    let vline_related = Scatter::new(vec![med_related, med_related], vec![0.0, LINE_HEIGHT])
        .mode(Mode::Lines)
        .name("Same Subject Pages median")
        .line(Line::new().color("red").width(1.0).dash(DashType::Dash))
        .opacity(0.6);

    plot.add_trace(vline_nonrelated);
    plot.add_trace(vline_related);

    let label_nonrelated = Scatter::new(vec![med_nonrelated], vec![LINE_HEIGHT * 0.95])
        .mode(Mode::Text)
        .text_array(vec![format!("{:.3}", med_nonrelated)])
        .show_legend(false)
        .marker(Marker::new().opacity(0.0));

    let label_related = Scatter::new(vec![med_related], vec![LINE_HEIGHT * 0.95])
        .mode(Mode::Text)
        .text_array(vec![format!("{:.3}", med_related)])
        .show_legend(false)
        .marker(Marker::new().opacity(0.0).color("red"));

    plot.add_trace(label_nonrelated);
    plot.add_trace(label_related);

    let layout = Layout::new()
        .title("Distance frequency histogram".to_string())
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
        .map(|entry| {
            let name_full = entry.get_name();
            name_full
                .rsplit('/')
                .next()
                .map(|s| s.to_string())
                .unwrap_or(name_full)
        })
        .collect::<Vec<String>>();
    let entries_grok = entries
        .into_iter()
        .filter(|e| e.page_type == "grok")
        .cloned()
        .collect::<Vec<dataset::Entry>>();
    let page_names_grok = entries_grok
        .iter()
        .map(|entry| {
            let name_full = entry.get_name();
            name_full
                .rsplit('/')
                .next()
                .map(|s| s.to_string())
                .unwrap_or(name_full)
        })
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
                "../../dataset/grokvswiki/page/Web_fiction.html",
            );
            let page1_g = kolmox::filter::content::grok::get_content(&page1_g).unwrap();
            let page1_w = read_from_file(
                "../../dataset/grokvswiki/wiki/Web_fiction.html",
            );
            let page1_w = kolmox::filter::content::wiki::get_content(&page1_w).unwrap();
            let result1 = compressor.get_distance(&page1_g, &page1_w);
            println!("Distance: {}", result1);
            assert_approx_eq!(result1, 0.96, 0.01);
            let page2_g = read_from_file(
                "../../dataset/grokvswiki/page/Arra_San_Agustin.html",
            );
            let page2_g = kolmox::filter::content::grok::get_content(&page2_g).unwrap();
            let result2 = compressor.get_distance(&page2_g, &page1_w);
            println!("Distance: {}", result2);
            assert_approx_eq!(result2, 0.98, 0.01);
        }

        #[test]
        fn test_plagiarism() {
            let page_wiki = read_from_file(
                "assets/wikipedia.txt",
            );
            let page_ruwiki = read_from_file(
                "assets/ruwiki.txt",
            );
            let zstd = kolmox::compress::zstd::CompressZstd::<NoCache>::recommended();
            let distance = zstd.get_distance(&page_wiki, &page_ruwiki);
            println!("Distance: {}", distance);
            assert_approx_eq!(distance, 0.07, 0.01);
        }
    }
}
