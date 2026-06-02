# Changelog: rabuf

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2026-05-31

### Added
- Final code review report (`docs/reviews/2026-05-31_code_review.4.md`) confirming optimizations and 100% safe Rust compliance.
- Comprehensive code review report (`docs/reviews/2026-05-30_code_review.2.md`) identifying critical feature-flag compatibility bugs.
- Initial comprehensive code review report (`docs/reviews/2026-05-30_code_review.1.md`).

### Fixed
- Compilation error when multiple overflow removal features are enabled (e.g., via `--all-features`) by prioritizing `buf_overf_rem_half` over `buf_overf_rem_all`.
- Typos in comments ("ramdom" -> "random", "syncronization" -> "synchronization").
- `SeekFrom::End(x)` implementation for positive `x` to correctly seek past the end of the file.
- `clippy::let_and_return` warning.
- `clippy::needless_range_loop` warning by using `enumerate()`.

### Changed
- Use `vec![]` macro in `buf_stats` and remove empty doc comments based on Clippy suggestions.
- Update LFU eviction strategy in `add_chunk` from O(N) linear scan to O(log N) using `BTreeSet` for improved performance on large caches.
- Update `write_u64_le_slice` and `write_u64_le_slice2` to avoid redundant memory allocations by writing data directly to the chunks.
- Use idiomatic `u32::next_power_of_two()` in `roundup_powerof2` and use `u32::is_power_of_two()` for assertions.
- Standardize internal method names (e.g., `read_exact_maybeslice_vec_inner` to `read_exact_maybeslice_inner`) for better consistency.
- Improve `set_len` to correctly handle chunk truncation, fix cache consistency issues, and zero-out stale data in partially truncated chunks.
- Use idiomatic `?` operator for error handling based on Clippy suggestions.
- Remove unnecessary `PhantomData` placeholders, commented-out dead code, and trailing `//--` comments for better maintainability.
- Skip unnecessary 0-filling in `Chunk::read_inplace` when the entire chunk is read.
- Highlight zero-copy usage in `read_exact_maybeslice` documentation.
- Move size check tests to `examples/check_size.rs` and add `make check-size` target.
- Suffix internal helper methods with `_inner` for better adherence to Rust naming conventions.
- Use idiomatic iterators in `set_len`.
- Use safe indexing in `set_len` and remove unnecessary `unsafe` blocks.
- Use safe indexing for all chunk access in `flush` and other methods instead of `unsafe` pointer arithmetic.
- Use safe slice indexing for `read_u8` instead of unsafe pointer arithmetic.
- Use safe indexing instead of `unsafe { get_unchecked(...) }`.
- Use safe slicing for `read_u16`, `read_u32`, and `read_u64` instead of unsafe slice creation.
- Use safe mutable slicing for write methods instead of unsafe mutable slice creation.
- Use safe indexing for binary search instead of unsafe pointer dereferencing.
- Use safe slicing for `read_u16_le`, `read_u32_le`, and `read_u64_le` instead of unsafe slice creation.
- Use safe indexing for byte-by-byte read loop instead of unsafe pointer arithmetic.
- Use safe slicing for `read_exact_small` instead of unsafe slice creation.
- Use safe idiomatic Rust for `write_u64_le_slice` and `write_u64_le_slice2` instead of unsafe pointer operations.
- Use safe indexing for chunk data length access instead of unsafe pointer-based access.
- Use safe idiomatic Rust for write methods instead of unsafe pointer operations.
- Use safe slicing for `read_exact_maybeslice` instead of unsafe slice creation.
- Use safe idiomatic Rust for `read_exact_maybeslice` instead of unsafe pointer operations.
- Use safe `from_le_bytes` conversion for `read_max_8_bytes` instead of unsafe integer parsing.
- Use safe mutable slicing for `read_exact_maybeslice` instead of unsafe mutable slice creation.

## [0.2.0] - 2025-09-25

### Added
- Specifications (`specs` folder).
- Additional tests.

### Fixed
- Infinite loop when reading at file sizes below `chunk` size.
- `clippy::unnecessary_cast` warning.

## [0.1.20] - 2024-06-09

### Fixed
- `clippy::suspicious_open_options` warning.

## [0.1.19] - 2023-02-12

### Added
- GitHub Action workflows for Ubuntu, macOS, and Windows.
- Test status badges in `README.tpl`.
- `MIRIFLAGS=-Zmiri-disable-isolation` for `cargo miri`.

### Changed
- Refactored `Makefile`.

### Removed
- `COPYING` file.

### Fixed
- `LICENSE-APACHE` and `LICENSE-MIT` files.

## [0.1.18] - 2023-01-28

### Added
- GitHub Action workflow for tests.
- Test status badges in `README.tpl`.

### Fixed
- Update rustc version from `1.66.0` to `1.66.1` in `Makefile`.
- `clippy::seek_to_start_instead_of_rewind` warning.
- Skip `test_size_of()` on Windows.

## [0.1.17] - 2023-01-10

### Added
- Version difference link into `CHANGELOG.md`.
- `rust-version = "1.56.0"` into `Cargo.toml`.
- `all-test-version` target into `Makefile`.
- Status badges into `README.tpl`.

### Changed
- Rename target `test-no_std` to `test-no-default-features` in `Makefile`.

### Removed
- Unused `bench-all` target from `Makefile`.

### Fixed
- Compilation error when using `--no-default-features`.
- `clippy::seek_to_start_instead_of_rewind` warning.

## [0.1.16] - 2023-01-05

### Fixed
- `clippy` warning regarding let-binding with unit value.

## [0.1.15] - 2022-06-13

### Changed
- Update to Rust 2021 edition.

## [0.1.14] - 2022-02-19

### Fixed
- Minimum supported rustc version set to `1.48.0`.

## [0.1.13] - 2022-02-01

### Changed
- Rewrite `Vec::get()` using pointer operations when `buf_debug` is disabled.

## [0.1.12] - 2022-01-26

### Added
- `write_u64_le_slice()` and `write_u64_le_slice2()` to `trait SmallWrite`.
- `into_vec()` to `enum MaybeSlice`.

## [0.1.11] - 2022-01-18

### Added
- `MyHasher` with a simple xorshift algorithm.

## [0.1.10] - 2022-01-14

### Added
- `buf_debug` feature for debugging `rabuf`.
- `write_u8()`, `write_u16_le()`, `write_u32_le()`, and `write_u64_le()`.
- `read_u16_le()`, `read_u32_le()`, and `read_u64_le()`.

### Changed
- Rename `read_one_byte()` to `read_u8()`.

## [0.1.9] - 2022-01-11

### Added
- `prepare()` method to `RaBuf<T>`.
- `buf_auto_buf_size` support in `add_chunk()` for better performance.

## [0.1.8] - 2022-01-08

### Added
- `buf_hash_turbo` feature for improved performance.

## [0.1.7] - 2022-01-07

### Fixed
- Performance improvements.

## [0.1.6] - 2021-12-19

### Added
- Name field to `struct rabuf` for debugging.
- `buf_print_hits` feature.

### Fixed
- Bugs in `setup_auto_buf_size()`.

### Removed
- `buf_idx_btreemap` feature.

## [0.1.5] - 2021-12-13

### Added
- `read_fill_buffer()`.

## [0.1.4] - 2021-12-05

### Added
- `buf_pin_zero` feature.
- `buf_auto_buf_size` feature.

### Fixed
- Creation methods of `struct RaBuf<T>`.

## [0.1.3] - 2021-11-26

### Added
- `buf_overf_rem_all` and `buf_overf_rem_half` features.

### Changed
- Rewrite `flush()` to write out in offset order.
- Rewrite overflow removal strategy to use half/all removal.

## [0.1.2] - 2021-11-17

### Added
- `buf_lru` and `buf_stats` features.

## [0.1.1] - 2021-11-11

### Added
- Comprehensive tests.
- `FileSetLen`, `FileSync`, `SmallRead`, and `SmallWrite` traits and implementations.

## [0.1.0] - 2021-11-10

### Added
- Initial release.

[Unreleased]: https://github.com/aki-akaguma/rabuf/compare/v0.3.0..HEAD
[0.3.0]: https://github.com/aki-akaguma/rabuf/compare/v0.2.0..v0.3.0
[0.2.0]: https://github.com/aki-akaguma/rabuf/compare/v0.1.20..v0.2.0
[0.1.20]: https://github.com/aki-akaguma/rabuf/compare/v0.1.19..v0.1.20
[0.1.19]: https://github.com/aki-akaguma/rabuf/compare/v0.1.18..v0.1.19
[0.1.18]: https://github.com/aki-akaguma/rabuf/compare/v0.1.17..v0.1.18
[0.1.17]: https://github.com/aki-akaguma/rabuf/compare/v0.1.16..v0.1.17
[0.1.16]: https://github.com/aki-akaguma/rabuf/compare/v0.1.15..v0.1.16
[0.1.15]: https://github.com/aki-akaguma/rabuf/compare/v0.1.14..v0.1.15
[0.1.14]: https://github.com/aki-akaguma/rabuf/compare/v0.1.13..v0.1.14
[0.1.13]: https://github.com/aki-akaguma/rabuf/compare/v0.1.12..v0.1.13
[0.1.12]: https://github.com/aki-akaguma/rabuf/compare/v0.1.11..v0.1.12
[0.1.11]: https://github.com/aki-akaguma/rabuf/compare/v0.1.10..v0.1.11
[0.1.10]: https://github.com/aki-akaguma/rabuf/compare/v0.1.9..v0.1.10
[0.1.9]: https://github.com/aki-akaguma/rabuf/compare/v0.1.8..v0.1.9
[0.1.8]: https://github.com/aki-akaguma/rabuf/compare/v0.1.7..v0.1.8
[0.1.7]: https://github.com/aki-akaguma/rabuf/compare/v0.1.6..v0.1.7
[0.1.6]: https://github.com/aki-akaguma/rabuf/compare/v0.1.5..v0.1.6
[0.1.5]: https://github.com/aki-akaguma/rabuf/compare/v0.1.4..v0.1.5
[0.1.4]: https://github.com/aki-akaguma/rabuf/compare/v0.1.3..v0.1.4
[0.1.3]: https://github.com/aki-akaguma/rabuf/compare/v0.1.2..v0.1.3
[0.1.2]: https://github.com/aki-akaguma/rabuf/compare/v0.1.1..v0.1.2
[0.1.1]: https://github.com/aki-akaguma/rabuf/compare/v0.1.0..v0.1.1
[0.1.0]: https://github.com/aki-akaguma/rabuf/releases/tag/v0.1.0
