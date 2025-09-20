#[cfg(test)]
mod test {
    use rabuf::BufFile;
    use std::fs::OpenOptions;
    use std::io::{Read, Seek, Write};

    macro_rules! base_dir {
        () => {
            "target/tmp"
        };
    }

    #[test]
    fn test_small_chunk_size() {
        std::fs::create_dir_all(base_dir!()).unwrap();
        let path = concat!(base_dir!(), "/test_small_chunk_size");

        let f = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(path)
            .unwrap();
        let mut bf = BufFile::with_capacity("tes", f, 2, 8).unwrap();

        let data = b"This is a test with a small chunk size.";
        bf.write_all(data).unwrap();

        bf.rewind().unwrap();

        let mut br = Vec::new();
        bf.read_to_end(&mut br).unwrap();

        assert_eq!(br, data);
    }

    #[test]
    fn test_file_not_multiple_of_chunk_size() {
        std::fs::create_dir_all(base_dir!()).unwrap();
        let path = concat!(base_dir!(), "/test_file_not_multiple_of_chunk_size");

        let f = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(path)
            .unwrap();
        let mut bf = BufFile::with_capacity("tes", f, 16, 4).unwrap();

        let data = vec![0u8; 50]; // Not a multiple of 16
        bf.write_all(&data).unwrap();

        bf.rewind().unwrap();

        let mut br = Vec::new();
        bf.read_to_end(&mut br).unwrap();

        assert_eq!(br, data);
    }

    #[test]
    fn test_clear() {
        std::fs::create_dir_all(base_dir!()).unwrap();
        let path = concat!(base_dir!(), "/test_clear");

        let bw = b"Data to be cleared";
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
            bf.clear().unwrap();
        }

        {
            let mut f = std::fs::File::open(path).unwrap();
            let mut br = Vec::new();
            f.read_to_end(&mut br).unwrap();
            assert_eq!(&br, bw);
        }
    }
}
