# rabuf

[![crate][crate-image]][crate-link]
[![Docs][docs-image]][docs-link]
![Rust Version][rustc-image]
![Apache2/MIT licensed][license-image]
[![Test ubu][test-ubuntu-image]][test-ubuntu-link]
[![Test mac][test-windows-image]][test-windows-link]
[![Test win][test-macos-image]][test-macos-link]

The Buffer for random access file.

When you read and write a file,
this read and write in `Chunk` units and reduce IO operation.

## Features

- random access
- `Chunk` units os io operation
- reduce os io operation
- support small size access accel.
- minimum support rustc 1.56.1 (59eed8a2a 2021-11-01)

## Examples

### Write, Seek, Read

```rust
use rabuf::BufFile;
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};

std::fs::create_dir_all("target/tmp").unwrap();

let path = "target/tmp/doc_test_1";
let bw = b"ABCEDFG\nhijklmn\n";

let f = OpenOptions::new().create(true)
    .read(true).write(true).open(path).unwrap();
let mut bf = BufFile::new("tes", f).unwrap();
bf.write_all(bw).unwrap();

bf.seek(SeekFrom::Start(0)).unwrap();

let mut br = vec![0u8; bw.len()];
bf.read_exact(&mut br).unwrap();
assert_eq!(&br, bw);
```

### Write, Close, Open, Read

```rust
use rabuf::BufFile;
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};

std::fs::create_dir_all("target/tmp").unwrap();
let path = "target/tmp/doc_test_2";

let bw = b"abcdefg\nHIJKLMN\n";
{
    let f = OpenOptions::new().create(true)
        .read(true).write(true).open(path).unwrap();
    let mut bf = BufFile::new("tes", f).unwrap();
    bf.write_all(bw).unwrap();
}
{
    let f = OpenOptions::new().create(true)
        .read(true).write(true).open(path).unwrap();
    let mut bf = BufFile::new("tes", f).unwrap();
    let mut br = vec![0u8; bw.len()];
    bf.read_exact(&mut br).unwrap();
    assert_eq!(&br, bw);
}
```

# Changelogs

[This crate's changelog here.](https://github.com/aki-akaguma/rabuf/blob/main/CHANGELOG.md)

# License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   https://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   https://opensource.org/licenses/MIT)

at your option.

[//]: # (badges)

[crate-image]: https://img.shields.io/crates/v/rabuf.svg
[crate-link]: https://crates.io/crates/rabuf
[docs-image]: https://docs.rs/rabuf/badge.svg
[docs-link]: https://docs.rs/rabuf/
[rustc-image]: https://img.shields.io/badge/rustc-1.56+-blue.svg
[license-image]: https://img.shields.io/badge/license-Apache2.0/MIT-blue.svg
[test-ubuntu-image]: https://github.com/aki-akaguma/rabuf/actions/workflows/test-ubuntu.yml/badge.svg
[test-ubuntu-link]: https://github.com/aki-akaguma/rabuf/actions/workflows/test-ubuntu.yml
[test-macos-image]: https://github.com/aki-akaguma/rabuf/actions/workflows/test-macos.yml/badge.svg
[test-macos-link]: https://github.com/aki-akaguma/rabuf/actions/workflows/test-macos.yml
[test-windows-image]: https://github.com/aki-akaguma/rabuf/actions/workflows/test-windows.yml/badge.svg
[test-windows-link]: https://github.com/aki-akaguma/rabuf/actions/workflows/test-windows.yml
