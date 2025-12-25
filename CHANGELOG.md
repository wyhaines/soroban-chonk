# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.0] - 2024-12-25

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
