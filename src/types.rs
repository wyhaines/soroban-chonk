use soroban_sdk::{Symbol, contracttype};

/// Storage keys for chunked content
#[derive(Clone)]
#[contracttype]
pub enum ChonkKey {
    /// Metadata for a content collection: collection_id -> ChonkMeta
    Meta(Symbol),
    /// Individual chunk: (collection_id, index) -> Bytes
    Chunk(Symbol, u32),
}

/// Metadata about a chunked content collection
#[derive(Clone, Debug, Default, PartialEq)]
#[contracttype]
pub struct ChonkMeta {
    /// Number of chunks in this collection
    pub count: u32,
    /// Total size in bytes across all chunks
    pub total_bytes: u32,
    /// Version for optimistic locking (incremented on each write)
    pub version: u32,
}
