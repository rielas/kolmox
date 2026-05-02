# Kolmox — Agent Codebase Guide

## What this project does

Kolmox computes **Normalized Compression Distance (NCD)** between text/HTML documents using Brotli or Zstd compression. Primary use case: measuring structural similarity between HTML pages. Formula:

```
NCD(x, y) = (C(xy) − min(C(x), C(y))) / max(C(x), C(y))
```

Result is in [0, 1]: 0 = identical, 1 = completely different.

---

## Repository layout

```
kolmox/
├── justfile                  # Task runner (see Commands below)
├── dataset/                  # Test HTML datasets (~60 MB, committed)
│   ├── grokvswiki/           # Main dataset: Grokipedia vs Wikipedia pages
│   ├── amazon/, imdb/, ...   # Other datasets
├── ncd/                      # Rust workspace
│   ├── Cargo.toml            # Workspace root (members: src/lib, benchmarks)
│   ├── src/lib/              # kolmox library crate
│   │   ├── compress/         # Compressor trait + Brotli/Zstd impls + Cache
│   │   └── filter/           # HtmlFilter trait + content-specific filters
│   ├── benchmarks/           # CLI binary crate for running benchmarks
│   │   └── src/lib/
│   │       ├── bench_tests/  # Property tests (triangle inequality, etc.)
│   │       └── benchmarks/   # Benchmark implementations
│   └── notebooks/            # Jupyter notebooks (analysis)
└── fetcher/                  # Python utility: fetches HTML via Selenium
    ├── main.py
    ├── path.py
    └── tests/
```

---

## Core abstractions

### `Compressor` trait — [ncd/src/lib/compress/mod.rs](ncd/src/lib/compress/mod.rs)
Implementations: `CompressBrotli`, `CompressZstd`.
Key method: `get_distance(&self, page_a: &str, page_b: &str) -> f64`.
Caching is generic via the `Cache` associated type — use `InMemoryCache` for batch processing, `NoCache` for one-shot comparisons.

### `HtmlFilter` trait — [ncd/src/lib/filter/mod.rs](ncd/src/lib/filter/mod.rs)
Walks the DOM and rebuilds a normalised string. Implementations:
- `wiki::WikiFilter` — strips Wikipedia-specific noise
- `grok::GrokFilter` — strips Grokipedia-specific noise
- `strip_content::StripContent` — keeps structure only
- `filter_attributes::FilterAttributes` — removes specific attributes

---

## Commands

```bash
just build           # cargo build --release (ncd/)
just tests           # cargo test --release (lib + benchmarks)
just format          # cargo fmt (lib + benchmarks)
just wiki-vs-grok    # run wiki-vs-grok benchmark on grokvswiki dataset
just same-page       # run same-page benchmark on imdb dataset
just fetch-wikivsgrok  # fetch grokvswiki dataset via Python fetcher (uv run)
just optimal-opts    # sweep compression params on two page pairs
```
