#![no_std]

mod chonk;
mod iter;
mod types;

pub use chonk::Chonk;
pub use iter::ChonkIter;
pub use types::{ChonkKey, ChonkMeta};

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::{Chonk, ChonkIter, ChonkKey, ChonkMeta};
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use soroban_sdk::{Address, Bytes, Env, contract, contractimpl, symbol_short};

    #[contract]
    pub struct TestContract;

    #[contractimpl]
    impl TestContract {}

    fn setup_test_env() -> (Env, Address) {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        (env, contract_id)
    }

    fn bytes(env: &Env, data: &[u8]) -> Bytes {
        Bytes::from_slice(env, data)
    }

    #[test]
    fn test_empty_chonk() {
        let (env, contract_id) = setup_test_env();

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            assert_eq!(chonk.count(), 0);
            assert_eq!(chonk.total_bytes(), 0);
            assert!(chonk.is_empty());
            assert!(chonk.get(0).is_none());
        });
    }

    #[test]
    fn test_push_and_get() {
        let (env, contract_id) = setup_test_env();

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            let chunk1 = bytes(&env, b"Hello, ");
            let chunk2 = bytes(&env, b"World!");

            let idx1 = chonk.push(chunk1.clone());
            let idx2 = chonk.push(chunk2.clone());

            assert_eq!(idx1, 0);
            assert_eq!(idx2, 1);
            assert_eq!(chonk.count(), 2);
            assert_eq!(chonk.total_bytes(), 13);

            assert_eq!(chonk.get(0), Some(chunk1));
            assert_eq!(chonk.get(1), Some(chunk2));
            assert!(chonk.get(2).is_none());
        });
    }

    #[test]
    fn test_assemble() {
        let (env, contract_id) = setup_test_env();

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            chonk.push(bytes(&env, b"Hello, "));
            chonk.push(bytes(&env, b"World!"));

            let assembled = chonk.assemble();
            assert_eq!(assembled, bytes(&env, b"Hello, World!"));
        });
    }

    #[test]
    fn test_write_chunked() {
        let (env, contract_id) = setup_test_env();

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            let content = bytes(&env, b"ABCDEFGHIJ"); // 10 bytes
            chonk.write_chunked(content.clone(), 3);

            assert_eq!(chonk.count(), 4); // 3 + 3 + 3 + 1
            assert_eq!(chonk.get(0), Some(bytes(&env, b"ABC")));
            assert_eq!(chonk.get(1), Some(bytes(&env, b"DEF")));
            assert_eq!(chonk.get(2), Some(bytes(&env, b"GHI")));
            assert_eq!(chonk.get(3), Some(bytes(&env, b"J")));

            let assembled = chonk.assemble();
            assert_eq!(assembled, content);
        });
    }

    #[test]
    fn test_set() {
        let (env, contract_id) = setup_test_env();

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            chonk.push(bytes(&env, b"old"));
            chonk.set(0, bytes(&env, b"new_value"));

            assert_eq!(chonk.get(0), Some(bytes(&env, b"new_value")));
            assert_eq!(chonk.total_bytes(), 9);
        });
    }

    #[test]
    fn test_insert() {
        let (env, contract_id) = setup_test_env();

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            chonk.push(bytes(&env, b"A"));
            chonk.push(bytes(&env, b"C"));
            chonk.insert(1, bytes(&env, b"B"));

            assert_eq!(chonk.count(), 3);
            assert_eq!(chonk.get(0), Some(bytes(&env, b"A")));
            assert_eq!(chonk.get(1), Some(bytes(&env, b"B")));
            assert_eq!(chonk.get(2), Some(bytes(&env, b"C")));
        });
    }

    #[test]
    fn test_remove() {
        let (env, contract_id) = setup_test_env();

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            chonk.push(bytes(&env, b"A"));
            chonk.push(bytes(&env, b"B"));
            chonk.push(bytes(&env, b"C"));

            let removed = chonk.remove(1);

            assert_eq!(removed, Some(bytes(&env, b"B")));
            assert_eq!(chonk.count(), 2);
            assert_eq!(chonk.get(0), Some(bytes(&env, b"A")));
            assert_eq!(chonk.get(1), Some(bytes(&env, b"C")));
        });
    }

    #[test]
    fn test_clear() {
        let (env, contract_id) = setup_test_env();

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            chonk.push(bytes(&env, b"A"));
            chonk.push(bytes(&env, b"B"));
            chonk.clear();

            assert!(chonk.is_empty());
            assert_eq!(chonk.count(), 0);
            assert_eq!(chonk.total_bytes(), 0);
        });
    }

    #[test]
    fn test_iter() {
        let (env, contract_id) = setup_test_env();

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            chonk.push(bytes(&env, b"A"));
            chonk.push(bytes(&env, b"B"));
            chonk.push(bytes(&env, b"C"));

            let chunks: std::vec::Vec<Bytes> = chonk.iter().collect();

            assert_eq!(chunks.len(), 3);
            assert_eq!(chunks[0], bytes(&env, b"A"));
            assert_eq!(chunks[1], bytes(&env, b"B"));
            assert_eq!(chunks[2], bytes(&env, b"C"));
        });
    }

    #[test]
    fn test_append() {
        let (env, contract_id) = setup_test_env();

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            chonk.append(bytes(&env, b"Hello"), 20);
            assert_eq!(chonk.count(), 1);

            chonk.append(bytes(&env, b", World!"), 20);
            assert_eq!(chonk.count(), 1); // Should append to existing
            assert_eq!(chonk.get(0), Some(bytes(&env, b"Hello, World!")));

            chonk.append(bytes(&env, b" This is a long addition"), 20);
            assert_eq!(chonk.count(), 2); // Should create new chunk
        });
    }

    #[test]
    fn test_get_range() {
        let (env, contract_id) = setup_test_env();

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            for i in 0..10u8 {
                chonk.push(bytes(&env, &[b'0' + i]));
            }

            let range = chonk.get_range(3, 4);
            assert_eq!(range.len(), 4);
        });
    }

    #[test]
    fn test_multiple_collections() {
        let (env, contract_id) = setup_test_env();

        env.as_contract(&contract_id, || {
            let chonk_a = Chonk::open(&env, symbol_short!("a"));
            let chonk_b = Chonk::open(&env, symbol_short!("b"));

            chonk_a.push(bytes(&env, b"A content"));
            chonk_b.push(bytes(&env, b"B content"));

            assert_eq!(chonk_a.count(), 1);
            assert_eq!(chonk_b.count(), 1);
            assert_eq!(chonk_a.get(0), Some(bytes(&env, b"A content")));
            assert_eq!(chonk_b.get(0), Some(bytes(&env, b"B content")));
        });
    }

    #[test]
    fn test_version_tracking() {
        let (env, contract_id) = setup_test_env();

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            assert_eq!(chonk.meta().version, 0);

            chonk.push(bytes(&env, b"A"));
            assert_eq!(chonk.meta().version, 1);

            chonk.push(bytes(&env, b"B"));
            assert_eq!(chonk.meta().version, 2);

            chonk.set(0, bytes(&env, b"A2"));
            assert_eq!(chonk.meta().version, 3);

            chonk.remove(1);
            assert_eq!(chonk.meta().version, 4);
        });
    }

    // ─── Error Handling Tests ───────────────────────────────

    #[test]
    #[should_panic(expected = "Index out of bounds")]
    fn test_set_out_of_bounds_panics() {
        let (env, contract_id) = setup_test_env();

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));
            chonk.set(0, bytes(&env, b"data")); // No chunks exist
        });
    }

    #[test]
    #[should_panic(expected = "Index out of bounds")]
    fn test_set_beyond_count_panics() {
        let (env, contract_id) = setup_test_env();

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));
            chonk.push(bytes(&env, b"A"));
            chonk.set(5, bytes(&env, b"data")); // Index 5 doesn't exist
        });
    }

    #[test]
    #[should_panic(expected = "Index out of bounds")]
    fn test_insert_beyond_count_panics() {
        let (env, contract_id) = setup_test_env();

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));
            chonk.push(bytes(&env, b"A"));
            chonk.insert(5, bytes(&env, b"data")); // Index 5 is beyond count (1)
        });
    }

    #[test]
    fn test_remove_out_of_bounds_returns_none() {
        let (env, contract_id) = setup_test_env();

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));
            assert!(chonk.remove(0).is_none()); // Empty collection

            chonk.push(bytes(&env, b"A"));
            assert!(chonk.remove(5).is_none()); // Beyond count
        });
    }

    // ─── Empty Content & Boundary Tests ─────────────────────

    #[test]
    fn test_push_empty_bytes() {
        let (env, contract_id) = setup_test_env();

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            let empty = Bytes::new(&env);
            let idx = chonk.push(empty.clone());

            assert_eq!(idx, 0);
            assert_eq!(chonk.count(), 1);
            assert_eq!(chonk.total_bytes(), 0);
            assert_eq!(chonk.get(0), Some(empty));
        });
    }

    #[test]
    fn test_write_chunked_empty_content() {
        let (env, contract_id) = setup_test_env();

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            chonk.push(bytes(&env, b"existing"));
            assert_eq!(chonk.count(), 1);

            // Write empty content - should clear and leave empty
            chonk.write_chunked(Bytes::new(&env), 10);

            assert!(chonk.is_empty());
            assert_eq!(chonk.count(), 0);
        });
    }

    #[test]
    fn test_write_chunked_exact_chunk_size() {
        let (env, contract_id) = setup_test_env();

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            // Content exactly divisible by chunk_size
            let content = bytes(&env, b"ABCDEF"); // 6 bytes
            chonk.write_chunked(content.clone(), 3);

            assert_eq!(chonk.count(), 2); // 3 + 3
            assert_eq!(chonk.get(0), Some(bytes(&env, b"ABC")));
            assert_eq!(chonk.get(1), Some(bytes(&env, b"DEF")));
            assert_eq!(chonk.assemble(), content);
        });
    }

    #[test]
    fn test_write_chunked_content_smaller_than_chunk_size() {
        let (env, contract_id) = setup_test_env();

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            let content = bytes(&env, b"Hi"); // 2 bytes
            chonk.write_chunked(content.clone(), 100);

            assert_eq!(chonk.count(), 1);
            assert_eq!(chonk.get(0), Some(content));
        });
    }

    #[test]
    fn test_write_chunked_single_byte_chunks() {
        let (env, contract_id) = setup_test_env();

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            let content = bytes(&env, b"ABC");
            chonk.write_chunked(content.clone(), 1);

            assert_eq!(chonk.count(), 3);
            assert_eq!(chonk.get(0), Some(bytes(&env, b"A")));
            assert_eq!(chonk.get(1), Some(bytes(&env, b"B")));
            assert_eq!(chonk.get(2), Some(bytes(&env, b"C")));
            assert_eq!(chonk.assemble(), content);
        });
    }

    #[test]
    #[should_panic(expected = "chunk_size must be greater than 0")]
    fn test_write_chunked_zero_chunk_size_panics() {
        let (env, contract_id) = setup_test_env();

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));
            chonk.write_chunked(bytes(&env, b"data"), 0);
        });
    }

    #[test]
    fn test_insert_at_beginning() {
        let (env, contract_id) = setup_test_env();

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            chonk.push(bytes(&env, b"B"));
            chonk.push(bytes(&env, b"C"));
            chonk.insert(0, bytes(&env, b"A"));

            assert_eq!(chonk.count(), 3);
            assert_eq!(chonk.get(0), Some(bytes(&env, b"A")));
            assert_eq!(chonk.get(1), Some(bytes(&env, b"B")));
            assert_eq!(chonk.get(2), Some(bytes(&env, b"C")));
        });
    }

    #[test]
    fn test_insert_at_end() {
        let (env, contract_id) = setup_test_env();

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            chonk.push(bytes(&env, b"A"));
            chonk.push(bytes(&env, b"B"));
            chonk.insert(2, bytes(&env, b"C")); // Insert at count (end)

            assert_eq!(chonk.count(), 3);
            assert_eq!(chonk.get(0), Some(bytes(&env, b"A")));
            assert_eq!(chonk.get(1), Some(bytes(&env, b"B")));
            assert_eq!(chonk.get(2), Some(bytes(&env, b"C")));
        });
    }

    #[test]
    fn test_insert_into_empty() {
        let (env, contract_id) = setup_test_env();

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            chonk.insert(0, bytes(&env, b"A"));

            assert_eq!(chonk.count(), 1);
            assert_eq!(chonk.get(0), Some(bytes(&env, b"A")));
        });
    }

    #[test]
    fn test_remove_first_chunk() {
        let (env, contract_id) = setup_test_env();

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            chonk.push(bytes(&env, b"A"));
            chonk.push(bytes(&env, b"B"));
            chonk.push(bytes(&env, b"C"));

            let removed = chonk.remove(0);

            assert_eq!(removed, Some(bytes(&env, b"A")));
            assert_eq!(chonk.count(), 2);
            assert_eq!(chonk.get(0), Some(bytes(&env, b"B")));
            assert_eq!(chonk.get(1), Some(bytes(&env, b"C")));
        });
    }

    #[test]
    fn test_remove_last_chunk() {
        let (env, contract_id) = setup_test_env();

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            chonk.push(bytes(&env, b"A"));
            chonk.push(bytes(&env, b"B"));
            chonk.push(bytes(&env, b"C"));

            let removed = chonk.remove(2);

            assert_eq!(removed, Some(bytes(&env, b"C")));
            assert_eq!(chonk.count(), 2);
            assert_eq!(chonk.get(0), Some(bytes(&env, b"A")));
            assert_eq!(chonk.get(1), Some(bytes(&env, b"B")));
        });
    }

    #[test]
    fn test_remove_only_chunk() {
        let (env, contract_id) = setup_test_env();

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            chonk.push(bytes(&env, b"Only"));
            let removed = chonk.remove(0);

            assert_eq!(removed, Some(bytes(&env, b"Only")));
            assert!(chonk.is_empty());
            assert_eq!(chonk.count(), 0);
        });
    }

    // ─── Iterator Edge Case Tests ───────────────────────────

    #[test]
    fn test_iter_empty_collection() {
        let (env, contract_id) = setup_test_env();

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            let chunks: std::vec::Vec<Bytes> = chonk.iter().collect();
            assert!(chunks.is_empty());
        });
    }

    #[test]
    fn test_iter_exact_size() {
        let (env, contract_id) = setup_test_env();

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            chonk.push(bytes(&env, b"A"));
            chonk.push(bytes(&env, b"B"));
            chonk.push(bytes(&env, b"C"));

            let mut iter = chonk.iter();
            assert_eq!(iter.len(), 3);

            iter.next();
            assert_eq!(iter.len(), 2);

            iter.next();
            assert_eq!(iter.len(), 1);

            iter.next();
            assert_eq!(iter.len(), 0);

            // Exhausted iterator
            assert!(iter.next().is_none());
            assert_eq!(iter.len(), 0);
        });
    }

    #[test]
    fn test_iter_size_hint() {
        let (env, contract_id) = setup_test_env();

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            chonk.push(bytes(&env, b"A"));
            chonk.push(bytes(&env, b"B"));
            chonk.push(bytes(&env, b"C"));

            let mut iter = chonk.iter();
            assert_eq!(iter.size_hint(), (3, Some(3)));

            iter.next();
            assert_eq!(iter.size_hint(), (2, Some(2)));

            iter.next();
            iter.next();
            assert_eq!(iter.size_hint(), (0, Some(0)));
        });
    }

    #[test]
    fn test_iter_partial() {
        let (env, contract_id) = setup_test_env();

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            chonk.push(bytes(&env, b"A"));
            chonk.push(bytes(&env, b"B"));
            chonk.push(bytes(&env, b"C"));

            // Only take first 2
            let first_two: std::vec::Vec<Bytes> = chonk.iter().take(2).collect();

            assert_eq!(first_two.len(), 2);
            assert_eq!(first_two[0], bytes(&env, b"A"));
            assert_eq!(first_two[1], bytes(&env, b"B"));
        });
    }

    #[test]
    fn test_assemble_empty() {
        let (env, contract_id) = setup_test_env();

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            let assembled = chonk.assemble();
            assert_eq!(assembled.len(), 0);
        });
    }

    // ─── Types Tests ────────────────────────────────────────

    #[test]
    fn test_chonk_meta_default() {
        let meta: ChonkMeta = Default::default();

        assert_eq!(meta.count, 0);
        assert_eq!(meta.total_bytes, 0);
        assert_eq!(meta.version, 0);
    }

    #[test]
    fn test_chonk_meta_equality() {
        let meta1 = ChonkMeta::default();
        let meta2 = ChonkMeta::default();

        assert_eq!(meta1, meta2);

        let mut meta3 = ChonkMeta::default();
        meta3.count = 1;

        assert_ne!(meta1, meta3);
    }

    #[test]
    fn test_chonk_id() {
        let (env, contract_id) = setup_test_env();

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("myid"));

            assert_eq!(*chonk.id(), symbol_short!("myid"));
        });
    }

    // ─── Append Edge Case Tests ─────────────────────────────

    #[test]
    fn test_append_to_empty_collection() {
        let (env, contract_id) = setup_test_env();

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            chonk.append(bytes(&env, b"First"), 100);

            assert_eq!(chonk.count(), 1);
            assert_eq!(chonk.get(0), Some(bytes(&env, b"First")));
        });
    }

    #[test]
    fn test_append_exactly_at_max_chunk_size() {
        let (env, contract_id) = setup_test_env();

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            // First chunk of 5 bytes
            chonk.append(bytes(&env, b"Hello"), 10);

            // Add 5 more bytes - exactly reaches max_chunk_size
            chonk.append(bytes(&env, b"World"), 10);

            assert_eq!(chonk.count(), 1); // Should still fit in one chunk
            assert_eq!(chonk.get(0), Some(bytes(&env, b"HelloWorld")));
        });
    }

    #[test]
    fn test_append_exceeds_max_chunk_size() {
        let (env, contract_id) = setup_test_env();

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            // First chunk of 8 bytes
            chonk.append(bytes(&env, b"12345678"), 10);

            // Add 5 more bytes - would exceed max_chunk_size
            chonk.append(bytes(&env, b"ABCDE"), 10);

            assert_eq!(chonk.count(), 2);
            assert_eq!(chonk.get(0), Some(bytes(&env, b"12345678")));
            assert_eq!(chonk.get(1), Some(bytes(&env, b"ABCDE")));
        });
    }

    #[test]
    fn test_append_empty_bytes() {
        let (env, contract_id) = setup_test_env();

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            chonk.append(bytes(&env, b"Data"), 10);
            chonk.append(Bytes::new(&env), 10); // Append empty

            assert_eq!(chonk.count(), 1);
            assert_eq!(chonk.get(0), Some(bytes(&env, b"Data")));
        });
    }

    #[test]
    fn test_append_empty_bytes_to_empty_collection() {
        let (env, contract_id) = setup_test_env();

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            // Appending empty content to empty collection should be a no-op
            chonk.append(Bytes::new(&env), 10);

            assert!(chonk.is_empty());
            assert_eq!(chonk.count(), 0);
        });
    }

    // ─── Get Range Edge Case Tests ──────────────────────────

    #[test]
    fn test_get_range_empty_collection() {
        let (env, contract_id) = setup_test_env();

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            let range = chonk.get_range(0, 5);
            assert_eq!(range.len(), 0);
        });
    }

    #[test]
    fn test_get_range_start_beyond_count() {
        let (env, contract_id) = setup_test_env();

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            chonk.push(bytes(&env, b"A"));
            chonk.push(bytes(&env, b"B"));

            let range = chonk.get_range(10, 5); // Start way past end
            assert_eq!(range.len(), 0);
        });
    }

    #[test]
    fn test_get_range_zero_count() {
        let (env, contract_id) = setup_test_env();

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            chonk.push(bytes(&env, b"A"));
            chonk.push(bytes(&env, b"B"));

            let range = chonk.get_range(0, 0);
            assert_eq!(range.len(), 0);
        });
    }

    #[test]
    fn test_get_range_extends_past_end() {
        let (env, contract_id) = setup_test_env();

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            chonk.push(bytes(&env, b"A"));
            chonk.push(bytes(&env, b"B"));
            chonk.push(bytes(&env, b"C"));

            // Request more than available
            let range = chonk.get_range(1, 100);
            assert_eq!(range.len(), 2); // Only B and C
        });
    }

    #[test]
    fn test_get_range_exact_bounds() {
        let (env, contract_id) = setup_test_env();

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            chonk.push(bytes(&env, b"A"));
            chonk.push(bytes(&env, b"B"));
            chonk.push(bytes(&env, b"C"));

            let range = chonk.get_range(0, 3);
            assert_eq!(range.len(), 3);
        });
    }

    #[test]
    fn test_get_range_u32_max_overflow() {
        let (env, contract_id) = setup_test_env();

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            chonk.push(bytes(&env, b"A"));
            chonk.push(bytes(&env, b"B"));

            // start + count would overflow u32; should not panic
            let range = chonk.get_range(u32::MAX, 1);
            assert_eq!(range.len(), 0);
        });
    }

    // ─── Total Bytes Tracking Tests ─────────────────────────

    #[test]
    fn test_total_bytes_after_operations() {
        let (env, contract_id) = setup_test_env();

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            assert_eq!(chonk.total_bytes(), 0);

            // Push 5 bytes
            chonk.push(bytes(&env, b"Hello"));
            assert_eq!(chonk.total_bytes(), 5);

            // Push 6 more bytes
            chonk.push(bytes(&env, b"World!"));
            assert_eq!(chonk.total_bytes(), 11);

            // Set first chunk to 3 bytes (was 5)
            chonk.set(0, bytes(&env, b"Hi!"));
            assert_eq!(chonk.total_bytes(), 9); // 3 + 6

            // Insert 2 bytes
            chonk.insert(1, bytes(&env, b"XX"));
            assert_eq!(chonk.total_bytes(), 11); // 3 + 2 + 6

            // Remove middle chunk (2 bytes)
            chonk.remove(1);
            assert_eq!(chonk.total_bytes(), 9); // 3 + 6
        });
    }

    // ─── Version Tracking Additional Tests ──────────────────

    #[test]
    fn test_version_increments_on_insert() {
        let (env, contract_id) = setup_test_env();

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            chonk.push(bytes(&env, b"A"));
            let v1 = chonk.meta().version;

            chonk.insert(0, bytes(&env, b"B"));
            let v2 = chonk.meta().version;

            assert_eq!(v2, v1 + 1);
        });
    }

    #[test]
    fn test_version_increments_on_append() {
        let (env, contract_id) = setup_test_env();

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            chonk.append(bytes(&env, b"Hello"), 100);
            let v1 = chonk.meta().version;

            // Append to existing chunk still increments version (via set)
            chonk.append(bytes(&env, b"World"), 100);
            let v2 = chonk.meta().version;

            assert!(v2 > v1);
        });
    }

    #[test]
    fn test_version_after_clear() {
        let (env, contract_id) = setup_test_env();

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            chonk.push(bytes(&env, b"A"));
            chonk.push(bytes(&env, b"B"));

            let version_before = chonk.meta().version;
            chonk.clear();

            // After clear, version should be preserved and incremented
            let meta = chonk.meta();
            assert_eq!(meta.count, 0);
            assert_eq!(meta.total_bytes, 0);
            assert_eq!(meta.version, version_before + 1);
        });
    }

    #[test]
    fn test_version_across_write_chunked() {
        let (env, contract_id) = setup_test_env();

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            chonk.push(bytes(&env, b"old"));
            let version_before = chonk.meta().version; // 1

            // write_chunked calls clear() then 2x push()
            // clear: version_before + 1, push "ABC": +1, push "DEF": +1 = version_before + 3
            chonk.write_chunked(bytes(&env, b"ABCDEF"), 3);

            let version_after = chonk.meta().version;
            assert_eq!(version_after, version_before + 3); // clear + 2 pushes
            assert_eq!(chonk.count(), 2);
        });
    }

    // ─── Reopening Collection Tests ─────────────────────────

    #[test]
    fn test_reopen_collection() {
        let (env, contract_id) = setup_test_env();

        env.as_contract(&contract_id, || {
            // First open - add data
            {
                let chonk = Chonk::open(&env, symbol_short!("test"));
                chonk.push(bytes(&env, b"Persistent"));
            }

            // Second open - data should still be there
            {
                let chonk = Chonk::open(&env, symbol_short!("test"));
                assert_eq!(chonk.count(), 1);
                assert_eq!(chonk.get(0), Some(bytes(&env, b"Persistent")));
            }
        });
    }

    // ─── Large Data Tests ───────────────────────────────────

    #[test]
    fn test_many_small_chunks() {
        let (env, contract_id) = setup_test_env();

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            // Push 40 small chunks (SDK v25 limits write entries to 50)
            for i in 0u8..40 {
                chonk.push(bytes(&env, &[i]));
            }

            assert_eq!(chonk.count(), 40);
            assert_eq!(chonk.total_bytes(), 40);

            // Verify some values
            assert_eq!(chonk.get(0), Some(bytes(&env, &[0u8])));
            assert_eq!(chonk.get(20), Some(bytes(&env, &[20u8])));
            assert_eq!(chonk.get(39), Some(bytes(&env, &[39u8])));
        });
    }

    #[test]
    fn test_write_chunked_larger_content() {
        let (env, contract_id) = setup_test_env();

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            // Create 1KB of data
            let mut data = [0u8; 1024];
            for (i, byte) in data.iter_mut().enumerate() {
                *byte = (i % 256) as u8;
            }

            let content = Bytes::from_slice(&env, &data);
            chonk.write_chunked(content.clone(), 100);

            // 1024 / 100 = 10 chunks of 100 + 1 chunk of 24
            assert_eq!(chonk.count(), 11);
            assert_eq!(chonk.total_bytes(), 1024);

            // Verify reassembly
            let assembled = chonk.assemble();
            assert_eq!(assembled, content);
        });
    }
}
