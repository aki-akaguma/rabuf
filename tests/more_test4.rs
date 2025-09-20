#[cfg(test)]
mod test {
    use rabuf::{BufFile, SmallWrite};
    use std::fs::OpenOptions;
    use std::io::{Read, Seek, Write};

    macro_rules! base_dir {
        () => {
            "target/tmp"
        };
    }

    #[test]
    fn test_write_u64_le_slice() {
        std::fs::create_dir_all(base_dir!()).unwrap();
        let path = concat!(base_dir!(), "/test_write_u64_le_slice");

        let f = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(path)
            .unwrap();
        let mut bf = BufFile::new("tes", f).unwrap();

        let data = vec![0x0102030405060708, 0x090a0b0c0d0e0f10];
        bf.write_u64_le_slice(&data).unwrap();

        bf.rewind().unwrap();

        let mut br = vec![0u8; 16];
        bf.read_exact(&mut br).unwrap();

        let mut expected_data = vec![];
        for &val in &data {
            expected_data.extend_from_slice(&val.to_le_bytes());
        }
        assert_eq!(br, expected_data);
    }

    #[test]
    fn test_write_u64_le_slice2() {
        std::fs::create_dir_all(base_dir!()).unwrap();
        let path = concat!(base_dir!(), "/test_write_u64_le_slice2");

        let f = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(path)
            .unwrap();
        let mut bf = BufFile::new("tes", f).unwrap();

        let data1 = vec![0x0102030405060708, 0x090a0b0c0d0e0f10];
        let data2 = vec![0x1112131415161718, 0x191a1b1c1d1e1f20];
        bf.write_u64_le_slice2(&data1, &data2).unwrap();

        bf.rewind().unwrap();

        let mut br = vec![0u8; 32];
        bf.read_exact(&mut br).unwrap();

        let mut expected_data = vec![];
        for &val in &data1 {
            expected_data.extend_from_slice(&val.to_le_bytes());
        }
        for &val in &data2 {
            expected_data.extend_from_slice(&val.to_le_bytes());
        }
        assert_eq!(br, expected_data);
    }

    #[test]
    fn test_flush() {
        std::fs::create_dir_all(base_dir!()).unwrap();
        let path = concat!(base_dir!(), "/test_flush");

        let bw = b"Data to be flushed";
        {
            let f = OpenOptions::new()
                .create(true)
                .truncate(true)
                .read(true)
                .write(true)
                .open(path)
                .unwrap();
            let mut bf = BufFile::new("tes", f).unwrap();
            bf.write_all(bw).unwrap();
            bf.flush().unwrap();
        }

        {
            let mut f = std::fs::File::open(path).unwrap();
            let mut br = vec![0u8; bw.len()];
            f.read_exact(&mut br).unwrap();
            assert_eq!(&br, bw);
        }
    }
}
