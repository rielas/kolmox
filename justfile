[working-directory: 'ncd']
build:
    cargo build --release

[working-directory: 'ncd']
all: build tests
    cargo run --release -- --help

[working-directory: 'ncd/benchmarks']
format-benchmark:
    cargo fmt

[working-directory: 'ncd']
format: format-benchmark
    cargo fmt

[working-directory: 'ncd']
tests: benchmark-tests
    cargo test --release

[working-directory: 'fetcher']
test-python:
    uv run pytest

test-all: tests test-python

[working-directory: 'ncd/benchmarks']
benchmark-tests:
    cargo test --release

[working-directory: 'ncd/benchmarks']
same-page:
    cargo run --release -- same-page ../../dataset/imdb/list/ls541382956/?ref_=tt_urls_2.html

[working-directory: 'ncd/benchmarks']
wiki-vs-grok:
    cargo run --release -- wiki-vs-grok ../../dataset/grokvswiki/dataset.csv

[working-directory: 'fetcher']
fetch-wikivsgrok:
    uv run main.py ../dataset/grokvswiki/

[working-directory: 'ncd/benchmarks']
optimal-opts:
    cargo run --bin benchmarks --release -- optimal-opts ../../dataset/grokvswiki/wiki/Web_fiction.html ../../dataset/grokvswiki/page/Web_fiction.html ../../dataset/grokvswiki/wiki/Arra_San_Agustin.html ../../dataset/grokvswiki/page/Arra_San_Agustin.html
