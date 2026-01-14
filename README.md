# Kolmox - Normalized Compression Distance in Rust

Kolmox is intended to measure the Normalized Compression Distance (NCD) between text documents using advanced compression algorithms like Brotli and Zstd. It was born as an effective way of measuring structure similarity between HTML pages, alternative to XML diff approach. It works faster and more accurately than existing alternatives like [html-similarity](https://pypi.org/project/html-similarity/) or [niteru](https://github.com/ninoseki/niteru).

# Distance Calculation

Normalized Compression Distance is calculated as:

$$
NCD(x, y) = \frac{C(xy) - \min(C(x), C(y))}{\max(C(x), C(y))}
$$

Where:
- $C(x)$ is the compressed size of text $x$
- $C(xy)$ is the compressed size of the concatenation of texts $x$ and $y$
- The result is normalized between 0 (identical) and 1 (completely different)

## Configuration

### Brotli Parameters

```rust
CompressBrotli::new(quality: u32, lg_window_size: u32)
// quality: 1-11 (higher = better compression, slower)
// lg_window_size: 10-24 (logarithmic window size)

CompressBrotli::recommended()      // quality=5, lg_window_size=21
CompressBrotli::max_quality()      // quality=11, lg_window_size=24
```

### Zstd Parameters

```rust
CompressZstd::recommended()        // Balanced speed/compression
```

## Examples

### Computing Distance Between Two Texts

```rust
use kolmox::compress::{brotli::CompressBrotli, Compressor};

let compressor = CompressBrotli::recommended();
let distance = compressor.get_distance(&text1, &text2);
println!("NCD: {:.4}", distance);
```

### Batch Processing with Cache

```rust
use kolmox::compress::{brotli::CompressBrotli, cache::InMemoryCache};

let compressor = CompressBrotli::<InMemoryCache>::recommended();
// Repeated comparisons reuse cached compression results
```

## References

- Normalized Compression Distance: https://en.wikipedia.org/wiki/Normalized_compression_distance
- Brotli: https://github.com/google/brotli
- Zstd: https://github.com/facebook/zstd
