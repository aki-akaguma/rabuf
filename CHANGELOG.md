# Changelog: rabuf

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
* `specs`
* more tests

### Fixed
* bug: Reading at file sizes below `chunk` size will result in an infinite loop.
* clippy: clippy::unnecessary_cast

## [0.1.20] (2024-06-09)
### Fixed
* clippy: clippy::suspicious_open_options

## [0.1.19] (2023-02-12)
### Added
* `.github/workflows/test-ubuntu.yml`
* `.github/workflows/test-macos.yml`
* `.github/workflows/test-windows.yml`
* test status badges into `README.tpl`
* `MIRIFLAGS=-Zmiri-disable-isolation` on `cargo miri`

### Changed
* refactored `Makefile`

### Removed
* `COPYING`

### Fixed
* `LICENSE-APACHE`, `LICENSE-MIT`

## [0.1.18] (2023-01-28)
### Added
* `.github/workflows/test.yml`
* test status badges into `README.tpl`

### Fixed
* Makefile: rustc version `1.66.0` to `1.66.1`
* clippy: `seek_to_start_instead_of_rewind`
* skip `test_size_of()` on windows

## [0.1.17] (2023-01-10)
### Added
* version difference link into `CHANGELOG.md`
* rust-version = "1.56.0" into Cargo.toml
* `all-test-version` target into Makefile
* badges into README.tpl

### Changed
* rename target `test-no_std` to `test-no-default-features` on Makefile

### Removed
* remove bench-all target from Makefile, no used

### Fixed
* bug: it can not be compiled at `--no-default-features`.
* clippy: https://rust-lang.github.io/rust-clippy/master/index.html#seek_to_start_instead_of_rewind

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

[Unreleased]: https://github.com/aki-akaguma/rabuf/compare/v0.1.20..HEAD
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
