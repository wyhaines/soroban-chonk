#![no_std]

mod chonk;
mod error;
mod iter;
mod types;

pub use chonk::Chonk;
pub use error::ChonkError;
pub use iter::ChonkIter;
pub use types::{ChonkKey, ChonkMeta};

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::{Chonk, ChonkError, ChonkIter, ChonkKey, ChonkMeta};
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use soroban_sdk::{Address, Bytes, Env, contract, contractimpl, symbol_short};

    // Test contract to provide contract context for storage access
    #[contract]
    pub struct TestContract;

    #[contractimpl]
    impl TestContract {
        pub fn init(_env: Env) {}
    }

    fn test_contract_id(env: &Env) -> Address {
        env.register(TestContract, ())
    }

    #[test]
    fn test_empty_chonk() {
        let env = Env::default();
        let contract_id = test_contract_id(&env);

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
        let env = Env::default();
        let contract_id = test_contract_id(&env);

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            let chunk1 = Bytes::from_slice(&env, b"Hello, ");
            let chunk2 = Bytes::from_slice(&env, b"World!");

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
        let env = Env::default();
        let contract_id = test_contract_id(&env);

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            chonk.push(Bytes::from_slice(&env, b"Hello, "));
            chonk.push(Bytes::from_slice(&env, b"World!"));

            let assembled = chonk.assemble();
            assert_eq!(assembled, Bytes::from_slice(&env, b"Hello, World!"));
        });
    }

    #[test]
    fn test_write_chunked() {
        let env = Env::default();
        let contract_id = test_contract_id(&env);

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            let content = Bytes::from_slice(&env, b"ABCDEFGHIJ"); // 10 bytes
            chonk.write_chunked(content.clone(), 3);

            assert_eq!(chonk.count(), 4); // 3 + 3 + 3 + 1
            assert_eq!(chonk.get(0), Some(Bytes::from_slice(&env, b"ABC")));
            assert_eq!(chonk.get(1), Some(Bytes::from_slice(&env, b"DEF")));
            assert_eq!(chonk.get(2), Some(Bytes::from_slice(&env, b"GHI")));
            assert_eq!(chonk.get(3), Some(Bytes::from_slice(&env, b"J")));

            let assembled = chonk.assemble();
            assert_eq!(assembled, content);
        });
    }

    #[test]
    fn test_set() {
        let env = Env::default();
        let contract_id = test_contract_id(&env);

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            chonk.push(Bytes::from_slice(&env, b"old"));
            chonk.set(0, Bytes::from_slice(&env, b"new_value"));

            assert_eq!(chonk.get(0), Some(Bytes::from_slice(&env, b"new_value")));
            assert_eq!(chonk.total_bytes(), 9);
        });
    }

    #[test]
    fn test_insert() {
        let env = Env::default();
        let contract_id = test_contract_id(&env);

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            chonk.push(Bytes::from_slice(&env, b"A"));
            chonk.push(Bytes::from_slice(&env, b"C"));
            chonk.insert(1, Bytes::from_slice(&env, b"B"));

            assert_eq!(chonk.count(), 3);
            assert_eq!(chonk.get(0), Some(Bytes::from_slice(&env, b"A")));
            assert_eq!(chonk.get(1), Some(Bytes::from_slice(&env, b"B")));
            assert_eq!(chonk.get(2), Some(Bytes::from_slice(&env, b"C")));
        });
    }

    #[test]
    fn test_remove() {
        let env = Env::default();
        let contract_id = test_contract_id(&env);

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            chonk.push(Bytes::from_slice(&env, b"A"));
            chonk.push(Bytes::from_slice(&env, b"B"));
            chonk.push(Bytes::from_slice(&env, b"C"));

            let removed = chonk.remove(1);

            assert_eq!(removed, Some(Bytes::from_slice(&env, b"B")));
            assert_eq!(chonk.count(), 2);
            assert_eq!(chonk.get(0), Some(Bytes::from_slice(&env, b"A")));
            assert_eq!(chonk.get(1), Some(Bytes::from_slice(&env, b"C")));
        });
    }

    #[test]
    fn test_clear() {
        let env = Env::default();
        let contract_id = test_contract_id(&env);

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            chonk.push(Bytes::from_slice(&env, b"A"));
            chonk.push(Bytes::from_slice(&env, b"B"));
            chonk.clear();

            assert!(chonk.is_empty());
            assert_eq!(chonk.count(), 0);
            assert_eq!(chonk.total_bytes(), 0);
        });
    }

    #[test]
    fn test_iter() {
        let env = Env::default();
        let contract_id = test_contract_id(&env);

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            chonk.push(Bytes::from_slice(&env, b"A"));
            chonk.push(Bytes::from_slice(&env, b"B"));
            chonk.push(Bytes::from_slice(&env, b"C"));

            let chunks: std::vec::Vec<Bytes> = chonk.iter().collect();

            assert_eq!(chunks.len(), 3);
            assert_eq!(chunks[0], Bytes::from_slice(&env, b"A"));
            assert_eq!(chunks[1], Bytes::from_slice(&env, b"B"));
            assert_eq!(chunks[2], Bytes::from_slice(&env, b"C"));
        });
    }

    #[test]
    fn test_append() {
        let env = Env::default();
        let contract_id = test_contract_id(&env);

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            chonk.append(Bytes::from_slice(&env, b"Hello"), 20);
            assert_eq!(chonk.count(), 1);

            chonk.append(Bytes::from_slice(&env, b", World!"), 20);
            assert_eq!(chonk.count(), 1); // Should append to existing
            assert_eq!(
                chonk.get(0),
                Some(Bytes::from_slice(&env, b"Hello, World!"))
            );

            chonk.append(Bytes::from_slice(&env, b" This is a long addition"), 20);
            assert_eq!(chonk.count(), 2); // Should create new chunk
        });
    }

    #[test]
    fn test_get_range() {
        let env = Env::default();
        let contract_id = test_contract_id(&env);

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            for i in 0..10u8 {
                let s = [b'0' + i];
                chonk.push(Bytes::from_slice(&env, &s));
            }

            let range = chonk.get_range(3, 4);
            assert_eq!(range.len(), 4);
        });
    }

    #[test]
    fn test_multiple_collections() {
        let env = Env::default();
        let contract_id = test_contract_id(&env);

        env.as_contract(&contract_id, || {
            let chonk_a = Chonk::open(&env, symbol_short!("a"));
            let chonk_b = Chonk::open(&env, symbol_short!("b"));

            chonk_a.push(Bytes::from_slice(&env, b"A content"));
            chonk_b.push(Bytes::from_slice(&env, b"B content"));

            assert_eq!(chonk_a.count(), 1);
            assert_eq!(chonk_b.count(), 1);
            assert_eq!(chonk_a.get(0), Some(Bytes::from_slice(&env, b"A content")));
            assert_eq!(chonk_b.get(0), Some(Bytes::from_slice(&env, b"B content")));
        });
    }

    #[test]
    fn test_version_tracking() {
        let env = Env::default();
        let contract_id = test_contract_id(&env);

        env.as_contract(&contract_id, || {
            let chonk = Chonk::open(&env, symbol_short!("test"));

            assert_eq!(chonk.meta().version, 0);

            chonk.push(Bytes::from_slice(&env, b"A"));
            assert_eq!(chonk.meta().version, 1);

            chonk.push(Bytes::from_slice(&env, b"B"));
            assert_eq!(chonk.meta().version, 2);

            chonk.set(0, Bytes::from_slice(&env, b"A2"));
            assert_eq!(chonk.meta().version, 3);

            chonk.remove(1);
            assert_eq!(chonk.meta().version, 4);
        });
    }
}
