# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.3](https://github.com/wyhaines/soroban-chonk/compare/v1.0.2...v1.0.3) - 2026-03-26

### Other

- Update soroban-sdk to v25 and regenerate snapshots
- Add test for version behavior through write_chunked
- Add O(n) storage cost warnings to insert() and remove() doc comments
- Remove unused ChonkError enum
- Panic on missing chunk data in ChonkIter::next() instead of returning None
- Add size_hint override to ChonkIter
- Fix clear() to preserve and increment version counter
- Harden append() with empty content guard and saturating_add
- Fix get_range integer overflow when start + count overflows u32
- Fix write_chunked infinite loop on zero chunk_size

## [1.0.2](https://github.com/wyhaines/soroban-chonk/compare/v1.0.1...v1.0.2) - 2026-01-22

### Other

- Fix clippy warnings.
- Commit code improvements.
- Add a bunch of tests.
- Only run release workflow if testing workflow succeeds.
- Update version from 1.0.0 to 1.0.1 in CHANGELOG

## [1.0.1] - 2024-12-25

### Added

- Initial release
- `Chonk` struct for managing chunked content collections
- Core operations: `push`, `get`, `set`, `insert`, `remove`, `clear`
- Batch operations: `write_chunked` for auto-chunking large content
- Smart append with `append` method
- Range queries with `get_range`
- Lazy iteration via `iter()` with `ExactSizeIterator` support
- Full content assembly with `assemble`
- `ChonkMeta` for tracking count, total_bytes, and version
- Multiple independent collections per contract via Symbol IDs
- Integration with soroban-render for progressive loading
