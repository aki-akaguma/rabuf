# rabuf

The Buffer for random access file.

When you read and write a file,
this read and write in `Chunk` units and reduce IO operation.

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
