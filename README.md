# soroban-chonk

[![CI](https://github.com/wyhaines/soroban-chonk/actions/workflows/ci.yml/badge.svg)](https://github.com/wyhaines/soroban-chonk/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/soroban-chonk.svg)](https://crates.io/crates/soroban-chonk)
[![License](https://img.shields.io/crates/l/soroban-chonk.svg)](LICENSE)

Chunked content storage for Soroban smart contracts.

Tip of the hat to [Nathan Toups](https://github.com/n2p5) for the [original name and idea](https://github.com/gnolang/gno/tree/master/examples/gno.land/p/n2p5/chonk) that this code is loosely inspired by.

## Overview

`soroban-chonk` provides a simple, efficient way to store large content as a series of chunks in Soroban contract storage. This is useful when:

- Content exceeds single storage entry limits (~64KB)
- You want to enable progressive loading of content
- You need to edit portions of large content without rewriting everything

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
soroban-chonk = "0.1"
```

## Quick Start

```rust
use soroban_chonk::prelude::*;
use soroban_sdk::{symbol_short, Bytes, Env};

// Open or create a chunk collection
let chonk = Chonk::open(&env, symbol_short!("article"));

// Add chunks
chonk.push(Bytes::from_slice(&env, b"Introduction..."));
chonk.push(Bytes::from_slice(&env, b"Chapter 1..."));
chonk.push(Bytes::from_slice(&env, b"Chapter 2..."));

// Or auto-chunk large content
let large_content = Bytes::from_slice(&env, &[0u8; 100_000]);
chonk.write_chunked(large_content, 4096); // 4KB chunks

// Read chunks
let first_chunk = chonk.get(0);
let meta = chonk.meta(); // { count: 25, total_bytes: 100000, version: 25 }

// Iterate
for chunk in chonk.iter() {
    // process chunk
}

// Assemble all chunks (be careful with large content!)
let full_content = chonk.assemble();
```

## API

### Chonk

| Method | Description |
|--------|-------------|
| `open(env, id)` | Create or open a chunk collection |
| `meta()` | Get metadata (count, total_bytes, version) |
| `count()` | Get number of chunks |
| `is_empty()` | Check if collection is empty |
| `get(index)` | Get a single chunk |
| `get_range(start, count)` | Get multiple chunks |
| `iter()` | Iterate over all chunks |
| `assemble()` | Combine all chunks into one Bytes |
| `push(data)` | Append a chunk |
| `set(index, data)` | Replace a chunk |
| `insert(index, data)` | Insert at position (shifts others) |
| `remove(index)` | Remove at position (shifts others) |
| `clear()` | Remove all chunks |
| `write_chunked(content, size)` | Auto-chunk content |
| `append(content, max_size)` | Smart append |

### ChonkMeta

Metadata about a chunk collection:

```rust
pub struct ChonkMeta {
    pub count: u32,        // Number of chunks
    pub total_bytes: u32,  // Total size across all chunks
    pub version: u32,      // Version (incremented on each write)
}
```

### ChonkKey

Storage keys used internally:

```rust
pub enum ChonkKey {
    Meta(Symbol),           // Metadata storage key
    Chunk(Symbol, u32),     // Individual chunk storage key
}
```

## Integration with soroban-render

For progressive content loading in smart contract UIs, see [soroban-render documentation](https://github.com/wyhaines/soroban-render).

Example with continuation markers:

```rust
use soroban_chonk::prelude::*;
use soroban_render_sdk::prelude::*;

pub fn render(env: Env) -> Bytes {
    let comments = Chonk::open(&env, symbol_short!("comments"));

    let mut builder = MarkdownBuilder::new(&env);

    // Render first 5 comments immediately
    for i in 0..5.min(comments.count()) {
        if let Some(chunk) = comments.get(i) {
            builder = builder.raw(chunk);
        }
    }

    // Add continuation marker for remaining comments
    if comments.count() > 5 {
        builder = builder.continuation("comments", 5, Some(comments.count()));
    }

    builder.build()
}
```

## License

[Apache-2.0](LICENSE)
