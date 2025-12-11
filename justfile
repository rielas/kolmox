[working-directory: 'ncd']
build:
    cargo build --release

[working-directory: 'ncd']
all: build
    cargo run --release
