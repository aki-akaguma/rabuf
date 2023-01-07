# Changelog: rabuf

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased] *
### Added
* version difference link into `CHANGELOG.md`
* rust-version = "1.56.0" into Cargo.toml

### Changed
* rename target `test-no_std` to `test-no-default-features` on Makefile

### Removed
* remove bench-all target from Makefile, no used

## [0.1.16] (2023-01-05)
### Fixed
* clippy: this let-binding has unit value

## [0.1.15] (2022-06-13)
### Changed
* changes to edition 2021

## [0.1.14] (2022-02-19)
### Fixed
* minimum support rustc `1.48.0`.

## [0.1.13] (2022-02-01)
### Changed
* `Vec::get()` has been rewritten by pointer operation in the case of NOT "buf_debug".

## [0.1.12] (2022-01-26)
### Added
* add `write_u64_le_slice()` and `write_u64_le_slice2()` to `trait SmallWrite`
* add `into_vec()` to `enum MaybeSlice`.

## [0.1.11] (2022-01-18)
### Added
* add `MyHasher` that has simple xorshift algorithm

## [0.1.10] (2022-01-14)
### Added
* add `buf_debug` to features for debugging `rabuf`.
* add `write_u8()`, `write_u16_le()`, `write_u32_le()` and `write_u64_le()`.
* add `read_u16_le()`, `read_u32_le()` and `read_u64_le()`.

### Changed
* rename `read_one_byte()` to `read_u8()`.

## [0.1.9] (2022-01-11)
### Added
* add `prepare()` method to `RaBuf<T>`.
* add `buf_auto_buf_size` support into `add_chunk()`, important performance.

## [0.1.8] (2022-01-08)
### Added
* add `buf_hash_turbo` to features. Important for Performance.

## [0.1.7] (2022-01-07)
### Fixed
* perforamance.

## [0.1.6] (2021-12-19)
### Added
* add name to `struct rabuf` for debugging.
* add `buf_print_hits` to features.

### Fixed
* fix some bugs of `setup_auto_buf_size()`.

### Removed
* remove `buf_idx_btreemap` from features.


## [0.1.5] (2021-12-13)
### Added
* add `read_fill_buffer()`.

## [0.1.4] (2021-12-05)
### Added
* add `buf_pin_zero` to features
* add `buf_auto_buf_size` to features

### Fixed
* bug: create methods of `struct RaBuf<T>`.

## [0.1.3] (2021-11-26)
### Added
* add `buf_overf_rem_all` and `buf_overf_rem_half` to features.

### Changed
* rewrite flush() method to be written out in the order of offset.
* rewrite the remove strategy at the over limit by the half/all remove.

## [0.1.2] (2021-11-17)
### Added
* add features: buf_lru, buf_stats

## [0.1.1] (2021-11-11)
### Added
* add tests
* add trait and impl: FileSetLen, FileSync, SmallRead, SmallWrite

## [0.1.0] (2021-11-10)
* first commit
