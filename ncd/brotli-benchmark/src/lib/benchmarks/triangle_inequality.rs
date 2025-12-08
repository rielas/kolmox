use super::get_dataset_path;
use crate::{benchmarks::Cache, dataset};
use core::brotli_default_filter_attributes;

pub fn triangle_inequality(cache: &mut Cache, dataset_name: &str) {
    use itertools::Itertools;

    let dataset =
        dataset::Dataset::new(get_dataset_path(dataset_name)).expect("Failed to load dataset");
    let entries = dataset.entries();

    for (a, b, c) in entries.iter().tuple_combinations() {
        let d_ab = cache.calculate(&a.get_content().unwrap(), &b.get_content().unwrap());
        let d_ac = cache.calculate(&a.get_content().unwrap(), &c.get_content().unwrap());
        let d_bc = cache.calculate(&b.get_content().unwrap(), &c.get_content().unwrap());

        assert!(
            d_ab + d_ac >= d_bc,
            "Triangle inequality d_ab + d_ac >= d_bc violated for a = {}, b = {}, c = {}",
            a.url,
            b.url,
            c.url
        );
        assert!(
            d_ab + d_bc >= d_ac,
            "Triangle inequality d_ab + d_bc >= d_ac violated for a = {}, b = {}, c = {}",
            a.url,
            b.url,
            c.url
        );
        assert!(
            d_ac + d_bc >= d_ab,
            "Triangle inequality d_ac + d_bc >= d_ab violated for a = {}, b = {}, c = {}",
            a.url,
            b.url,
            c.url
        );
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        benchmarks::get_dataset_path,
        dataset::{self, Dataset},
    };
    use core::brotli_default_filter_attributes;

    #[test]
    fn test_triangle_inequality() {
        let dataset =
            dataset::Dataset::new(get_dataset_path("imdb")).expect("Failed to load dataset");
        let page_a = dataset
            .get_content("https://www.imdb.com/title/tt0111161/?ref_=chttp_t_1")
            .unwrap();
        let page_b = dataset
            .get_content("https://www.imdb.com/video/vi2108539673/?ref_=ttvg_vi_5")
            .unwrap();
        let page_c = dataset
            .get_content("https://www.imdb.com/video/vi998032153/?ref_=ttvg_vi_3")
            .unwrap();
        let d_ac = brotli_default_filter_attributes(&page_a, &page_c);
        let d_ab = brotli_default_filter_attributes(&page_a, &page_b);
        let d_bc = brotli_default_filter_attributes(&page_b, &page_c);
        println!("d_ab = {d_ab}, d_ac = {d_ac}, d_bc = {d_bc}");
        assert!(d_ac + d_bc >= d_ab);
    }
}
