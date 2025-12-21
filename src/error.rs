use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ChonkError {
    /// Index is out of bounds
    IndexOutOfBounds = 1,
    /// Collection not found
    NotFound = 2,
    /// Chunk size exceeds maximum
    ChunkTooLarge = 3,
    /// Operation would exceed storage limits
    StorageLimitExceeded = 4,
}
