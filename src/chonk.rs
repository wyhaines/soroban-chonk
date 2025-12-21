use crate::iter::ChonkIter;
use crate::types::{ChonkKey, ChonkMeta};
use soroban_sdk::{Bytes, Env, Symbol, Vec};

/// A collection of chunked content stored in contract storage
pub struct Chonk<'a> {
    env: &'a Env,
    id: Symbol,
}

impl<'a> Chonk<'a> {
    /// Create or open a chunk collection
    pub fn open(env: &'a Env, id: Symbol) -> Self {
        Self { env, id }
    }

    /// Get the collection ID
    pub fn id(&self) -> &Symbol {
        &self.id
    }

    /// Get metadata for this collection
    pub fn meta(&self) -> ChonkMeta {
        let key = ChonkKey::Meta(self.id.clone());
        self.env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or_default()
    }

    /// Get number of chunks
    pub fn count(&self) -> u32 {
        self.meta().count
    }

    /// Get total bytes across all chunks
    pub fn total_bytes(&self) -> u32 {
        self.meta().total_bytes
    }

    /// Check if the collection is empty
    pub fn is_empty(&self) -> bool {
        self.count() == 0
    }

    // ─── Read Operations ───────────────────────────────────

    /// Get a single chunk by index
    pub fn get(&self, index: u32) -> Option<Bytes> {
        let key = ChonkKey::Chunk(self.id.clone(), index);
        self.env.storage().persistent().get(&key)
    }

    /// Get multiple chunks as a Vec
    pub fn get_range(&self, start: u32, count: u32) -> Vec<Bytes> {
        let mut result = Vec::new(self.env);
        let meta = self.meta();
        let end = core::cmp::min(start + count, meta.count);

        for i in start..end {
            if let Some(chunk) = self.get(i) {
                result.push_back(chunk);
            }
        }
        result
    }

    /// Iterate over all chunks
    pub fn iter(&self) -> ChonkIter<'_> {
        ChonkIter::new(self.env, self.id.clone(), self.count())
    }

    /// Assemble all chunks into a single Bytes
    /// Warning: May hit execution limits for very large content
    pub fn assemble(&self) -> Bytes {
        let mut result = Bytes::new(self.env);
        for chunk in self.iter() {
            result.append(&chunk);
        }
        result
    }

    // ─── Write Operations ──────────────────────────────────

    /// Save metadata
    fn save_meta(&self, meta: &ChonkMeta) {
        let key = ChonkKey::Meta(self.id.clone());
        self.env.storage().persistent().set(&key, meta);
    }

    /// Append a chunk to the end, returns the new index
    pub fn push(&self, data: Bytes) -> u32 {
        let mut meta = self.meta();
        let index = meta.count;

        let key = ChonkKey::Chunk(self.id.clone(), index);
        let data_len = data.len();
        self.env.storage().persistent().set(&key, &data);

        meta.count += 1;
        meta.total_bytes += data_len;
        meta.version += 1;
        self.save_meta(&meta);

        index
    }

    /// Replace a specific chunk
    pub fn set(&self, index: u32, data: Bytes) {
        let mut meta = self.meta();
        if index >= meta.count {
            panic!("Index out of bounds");
        }

        let key = ChonkKey::Chunk(self.id.clone(), index);

        // Adjust total_bytes
        if let Some(old_data) = self.env.storage().persistent().get::<_, Bytes>(&key) {
            meta.total_bytes -= old_data.len();
        }
        meta.total_bytes += data.len();
        meta.version += 1;

        self.env.storage().persistent().set(&key, &data);
        self.save_meta(&meta);
    }

    /// Insert a chunk at index (shifts subsequent chunks)
    pub fn insert(&self, index: u32, data: Bytes) {
        let mut meta = self.meta();
        if index > meta.count {
            panic!("Index out of bounds");
        }

        // Shift chunks from end to index
        for i in (index..meta.count).rev() {
            let from_key = ChonkKey::Chunk(self.id.clone(), i);
            let to_key = ChonkKey::Chunk(self.id.clone(), i + 1);
            if let Some(chunk) = self.env.storage().persistent().get::<_, Bytes>(&from_key) {
                self.env.storage().persistent().set(&to_key, &chunk);
            }
        }

        // Insert new chunk
        let key = ChonkKey::Chunk(self.id.clone(), index);
        let data_len = data.len();
        self.env.storage().persistent().set(&key, &data);

        meta.count += 1;
        meta.total_bytes += data_len;
        meta.version += 1;
        self.save_meta(&meta);
    }

    /// Remove a chunk at index (shifts subsequent chunks)
    pub fn remove(&self, index: u32) -> Option<Bytes> {
        let mut meta = self.meta();
        if index >= meta.count {
            return None;
        }

        // Get the chunk being removed
        let key = ChonkKey::Chunk(self.id.clone(), index);
        let removed: Option<Bytes> = self.env.storage().persistent().get(&key);

        // Shift subsequent chunks
        for i in index..(meta.count - 1) {
            let from_key = ChonkKey::Chunk(self.id.clone(), i + 1);
            let to_key = ChonkKey::Chunk(self.id.clone(), i);
            if let Some(chunk) = self.env.storage().persistent().get::<_, Bytes>(&from_key) {
                self.env.storage().persistent().set(&to_key, &chunk);
            }
        }

        // Remove last slot
        let last_key = ChonkKey::Chunk(self.id.clone(), meta.count - 1);
        self.env.storage().persistent().remove(&last_key);

        // Update metadata
        if let Some(ref data) = removed {
            meta.total_bytes -= data.len();
        }
        meta.count -= 1;
        meta.version += 1;
        self.save_meta(&meta);

        removed
    }

    /// Remove all chunks
    pub fn clear(&self) {
        let meta = self.meta();

        // Remove all chunks
        for i in 0..meta.count {
            let key = ChonkKey::Chunk(self.id.clone(), i);
            self.env.storage().persistent().remove(&key);
        }

        // Remove metadata
        let meta_key = ChonkKey::Meta(self.id.clone());
        self.env.storage().persistent().remove(&meta_key);
    }

    // ─── Bulk Operations ───────────────────────────────────

    /// Write content, automatically chunking at specified size
    pub fn write_chunked(&self, content: Bytes, chunk_size: u32) {
        // Clear existing content
        self.clear();

        let content_len = content.len();
        if content_len == 0 {
            return;
        }

        let mut offset = 0u32;
        while offset < content_len {
            let end = core::cmp::min(offset + chunk_size, content_len);
            let chunk = content.slice(offset..end);
            self.push(chunk);
            offset = end;
        }
    }

    /// Append content to last chunk or create new if it would exceed max size
    pub fn append(&self, content: Bytes, max_chunk_size: u32) {
        let meta = self.meta();

        if meta.count == 0 {
            self.push(content);
            return;
        }

        let last_index = meta.count - 1;
        if let Some(last_chunk) = self.get(last_index) {
            let new_len = last_chunk.len() + content.len();
            if new_len <= max_chunk_size {
                // Append to existing chunk
                let mut combined = Bytes::new(self.env);
                combined.append(&last_chunk);
                combined.append(&content);
                self.set(last_index, combined);
            } else {
                // Create new chunk
                self.push(content);
            }
        } else {
            self.push(content);
        }
    }
}
