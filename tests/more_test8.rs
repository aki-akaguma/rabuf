#[cfg(test)]
mod test {
    use rabuf::BufFile;
    use std::fs::OpenOptions;
    use std::io::{Read, Seek, SeekFrom, Write};

    macro_rules! base_dir {
        () => {
            "target/tmp"
        };
    }

    #[test]
    fn test_read_fill_buffer_empty_file() {
        std::fs::create_dir_all(base_dir!()).unwrap();
        let path = concat!(base_dir!(), "/test_read_fill_buffer_empty_file");

        let f = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(path)
            .unwrap();
        let mut bf = BufFile::new("tes", f).unwrap();

        bf.read_fill_buffer().unwrap();

        let mut br = Vec::new();
        let result = bf.read_to_end(&mut br).unwrap();

        assert_eq!(result, 0);
        assert!(br.is_empty());
    }

    #[test]
    fn test_read_fill_buffer_small_file() {
        std::fs::create_dir_all(base_dir!()).unwrap();
        let path = concat!(base_dir!(), "/test_read_fill_buffer_small_file");

        let f = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(path)
            .unwrap();
        let mut bf = BufFile::with_capacity("tes", f, 1024, 4).unwrap();

        let data = b"small file";
        bf.write_all(data).unwrap();

        bf.clear().unwrap();
        bf.read_fill_buffer().unwrap();

        bf.rewind().unwrap();

        let mut br = Vec::new();
        bf.read_to_end(&mut br).unwrap();

        assert_eq!(br, data);
    }

    #[test]
    fn test_seek_negative_from_current() {
        std::fs::create_dir_all(base_dir!()).unwrap();
        let path = concat!(base_dir!(), "/test_seek_negative_from_current");

        let f = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(path)
            .unwrap();
        let mut bf = BufFile::new("tes", f).unwrap();

        let data = b"0123456789";
        bf.write_all(data).unwrap();

        bf.seek(SeekFrom::Start(8)).unwrap();
        bf.seek(SeekFrom::Current(-4)).unwrap();

        let mut br = vec![0u8; 4];
        bf.read_exact(&mut br).unwrap();

        assert_eq!(br, &data[4..8]);
    }
}
