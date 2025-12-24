# soroban-chonk

Chunked content storage library for Soroban contracts. Stores large content across multiple storage entries with automatic chunking and reassembly.

## CONSTRAINTS

- `#![no_std]` environment
- Uses persistent storage
- Each chunk stored as separate storage entry
- Suitable for content exceeding single storage limits

---

## EXPORTS

```rust
use soroban_chonk::prelude::*;
// or
use soroban_chonk::{Chonk, ChonkError, ChonkIter, ChonkKey, ChonkMeta};
```

---

## CHONK

Main interface for chunked content collections.

### Constructor

```rust
let chonk = Chonk::open(env: &Env, id: Symbol);
```

BEHAVIOR: Opens or creates a collection. Does not allocate storage until first write.

---

### Metadata

| Method | Return | Description |
|--------|--------|-------------|
| `id()` | `&Symbol` | Collection identifier |
| `meta()` | `ChonkMeta` | Metadata struct |
| `count()` | `u32` | Number of chunks |
| `total_bytes()` | `u32` | Total bytes across all chunks |
| `is_empty()` | `bool` | True if count == 0 |

---

### Read Operations

| Method | Signature | Description |
|--------|-----------|-------------|
| `get` | `(index: u32) -> Option<Bytes>` | Get single chunk |
| `get_range` | `(start: u32, count: u32) -> Vec<Bytes>` | Get multiple chunks |
| `iter` | `() -> ChonkIter` | Iterate all chunks |
| `assemble` | `() -> Bytes` | Concatenate all chunks |

WARNING: `assemble()` may hit execution limits for large content.

---

### Write Operations

| Method | Signature | Description |
|--------|-----------|-------------|
| `push` | `(data: Bytes) -> u32` | Append chunk, returns index |
| `set` | `(index: u32, data: Bytes)` | Replace chunk at index |
| `insert` | `(index: u32, data: Bytes)` | Insert at index (shifts subsequent) |
| `remove` | `(index: u32) -> Option<Bytes>` | Remove at index (shifts subsequent) |
| `clear` | `()` | Remove all chunks and metadata |

PANICS: `set` panics if index >= count. `insert` panics if index > count.

---

### Bulk Operations

| Method | Signature | Description |
|--------|-----------|-------------|
| `write_chunked` | `(content: Bytes, chunk_size: u32)` | Clear and write with auto-chunking |
| `append` | `(content: Bytes, max_chunk_size: u32)` | Append to last chunk or create new |

`write_chunked` BEHAVIOR:
1. Clears existing content
2. Splits content into chunks of `chunk_size`
3. Stores each chunk

`append` BEHAVIOR:
1. If empty, push new chunk
2. If last chunk + content <= max_chunk_size, append to last chunk
3. Otherwise, push new chunk

---

## CHUNKMETA

```rust
pub struct ChonkMeta {
    pub count: u32,       // Number of chunks
    pub total_bytes: u32, // Total bytes
    pub version: u32,     // Incremented on each write
}
```

VERSION: Incremented on `push`, `set`, `insert`, `remove`. Useful for optimistic locking.

---

## CHONKKEY

Storage key enum:

```rust
pub enum ChonkKey {
    Meta(Symbol),         // Metadata: collection_id -> ChonkMeta
    Chunk(Symbol, u32),   // Chunk: (collection_id, index) -> Bytes
}
```

---

## CHONKITER

Iterator over chunks. Implements `Iterator<Item = Bytes>`.

```rust
for chunk in chonk.iter() {
    // process chunk
}

let chunks: Vec<Bytes> = chonk.iter().collect();
```

---

## USAGE PATTERNS

### Store Large Content

```rust
let chonk = Chonk::open(&env, symbol_short!("content"));
chonk.write_chunked(large_content, 4096); // 4KB chunks
```

### Retrieve Content

```rust
let chonk = Chonk::open(&env, symbol_short!("content"));
let full = chonk.assemble();
```

### Progressive Loading

```rust
let chonk = Chonk::open(&env, symbol_short!("content"));
let first_chunk = chonk.get(0);
let meta = chonk.meta();
// Return first chunk + continuation tag for remaining
```

### Multiple Collections

```rust
let headers = Chonk::open(&env, symbol_short!("headers"));
let body = Chonk::open(&env, symbol_short!("body"));
let footer = Chonk::open(&env, symbol_short!("footer"));
```

### Append Content

```rust
let chonk = Chonk::open(&env, symbol_short!("log"));
chonk.append(new_entry, 8192); // Append, max 8KB per chunk
```

---

## INTEGRATION WITH SOROBAN-RENDER

soroban-chonk provides storage for the `{{chunk}}` and `{{continue}}` protocols:

```rust
// Contract method called by viewer
pub fn get_chunk(env: Env, collection: Symbol, index: u32) -> Option<Bytes> {
    Chonk::open(&env, collection).get(index)
}

// Render method returns continuation tag
pub fn render(env: Env, path: Option<String>, viewer: Option<Address>) -> Bytes {
    let chonk = Chonk::open(&env, symbol_short!("page"));
    let first = chonk.get(0).unwrap_or(Bytes::new(&env));
    let meta = chonk.meta();

    if meta.count > 1 {
        // Include continuation tag
        MarkdownBuilder::new(&env)
            .raw(first)
            .continuation("page", 1, Some(meta.count))
            .build()
    } else {
        first
    }
}
```

---

## ERROR HANDLING

| Error | When |
|-------|------|
| `set` panic | index >= count |
| `insert` panic | index > count |
| `get` returns None | index >= count |
| `remove` returns None | index >= count |
