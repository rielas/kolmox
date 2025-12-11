[working-directory: 'ncd']
build:
    cargo build --release

[working-directory: 'ncd']
all: build
    cargo run --release -- --help

[working-directory: 'ncd/brotli-benchmark']
format-benchmark:
    cargo fmt

[working-directory: 'ncd']
format: format-benchmark
    cargo fmt

[working-directory: 'ncd']
tests: benchmark-tests
    cargo test --release

[working-directory: 'ncd/brotli-benchmark']
benchmark-tests:
    cargo test --release

[working-directory: 'ncd/brotli-benchmark']
same-page:
    cargo run --release -- same-page ../../dataset/imdb/list/ls541382956/?ref_=tt_urls_2.html

[working-directory: 'ncd/brotli-benchmark']
wiki-vs-grok:
    cargo run --release -- wiki-vs-grok ../../dataset/grokvswiki/dataset.csv

[working-directory: 'fetcher']
fetch-wikivsgrok:
    uv run main.py ../dataset/grokvswiki/

[working-directory: 'ncd/brotli-benchmark']
optimal-opts:
    cargo run --bin benchmark --release -- optimal-opts ../../dataset/grokvswiki/wiki/Web_fiction.html ../../dataset/grokvswiki/page/Web_fiction.html ../../dataset/grokvswiki/wiki/Arra_San_Agustin.html ../../dataset/grokvswiki/page/Arra_San_Agustin.html
