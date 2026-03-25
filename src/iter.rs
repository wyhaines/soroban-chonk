use crate::types::ChonkKey;
use soroban_sdk::{Bytes, Env, Symbol};

/// Iterator over chunks in a Chonk collection
pub struct ChonkIter<'a> {
    env: &'a Env,
    id: Symbol,
    count: u32,
    current: u32,
}

impl<'a> ChonkIter<'a> {
    pub fn new(env: &'a Env, id: Symbol, count: u32) -> Self {
        Self {
            env,
            id,
            count,
            current: 0,
        }
    }
}

impl<'a> Iterator for ChonkIter<'a> {
    type Item = Bytes;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current >= self.count {
            return None;
        }

        let key = ChonkKey::Chunk(self.id.clone(), self.current);
        self.current += 1;
        Some(
            self.env
                .storage()
                .persistent()
                .get(&key)
                .expect("Chunk missing from storage: data corruption"),
        )
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.count - self.current) as usize;
        (remaining, Some(remaining))
    }
}

impl<'a> ExactSizeIterator for ChonkIter<'a> {
    fn len(&self) -> usize {
        (self.count - self.current) as usize
    }
}
