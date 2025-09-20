#[cfg(test)]
mod test {
    use rabuf::{BufFile, SmallRead, SmallWrite};
    use std::fs::OpenOptions;
    use std::io::{Read, Seek, Write};

    macro_rules! base_dir {
        () => {
            "target/tmp"
        };
    }

    #[test]
    fn test_large_data() {
        std::fs::create_dir_all(base_dir!()).unwrap();
        let path = concat!(base_dir!(), "/test_large_data");

        let f = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(path)
            .unwrap();
        let mut bf = BufFile::with_capacity("tes", f, 1024, 4).unwrap();

        let data_size = 8192;
        let mut data = Vec::with_capacity(data_size);
        for i in 0..data_size {
            data.push((i % 256) as u8);
        }

        bf.write_all(&data).unwrap();

        bf.rewind().unwrap();

        let mut br = Vec::new();
        bf.read_to_end(&mut br).unwrap();

        assert_eq!(br, data);
    }

    #[test]
    fn test_read_max_8_bytes() {
        std::fs::create_dir_all(base_dir!()).unwrap();
        let path = concat!(base_dir!(), "/test_read_max_8_bytes");

        let f = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(path)
            .unwrap();
        let mut bf = BufFile::new("tes", f).unwrap();

        let val = 0x0102030405060708u64;
        bf.write_all(&val.to_le_bytes()).unwrap();

        bf.rewind().unwrap();
        assert_eq!(bf.read_max_8_bytes(8).unwrap(), val);

        bf.rewind().unwrap();
        assert_eq!(bf.read_max_8_bytes(4).unwrap(), 0x05060708);
    }

    #[test]
    fn test_read_exact_small() {
        std::fs::create_dir_all(base_dir!()).unwrap();
        let path = concat!(base_dir!(), "/test_read_exact_small");

        let f = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(path)
            .unwrap();
        let mut bf = BufFile::new("tes", f).unwrap();

        let data = b"small test data";
        bf.write_all(data).unwrap();

        bf.rewind().unwrap();

        let mut br = vec![0u8; data.len()];
        bf.read_exact_small(&mut br).unwrap();

        assert_eq!(br, data);
    }

    #[test]
    fn test_write_all_small() {
        std::fs::create_dir_all(base_dir!()).unwrap();
        let path = concat!(base_dir!(), "/test_write_all_small");

        let f = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(path)
            .unwrap();
        let mut bf = BufFile::new("tes", f).unwrap();

        let data = b"small test data";
        bf.write_all_small(data).unwrap();

        bf.rewind().unwrap();

        let mut br = vec![0u8; data.len()];
        bf.read_exact(&mut br).unwrap();

        assert_eq!(br, data);
    }
}
