#[cfg(test)]
mod test {
    use rabuf::{BufFile, FileSetLen};
    use std::fs::OpenOptions;
    use std::io::{Read, Seek, Write};

    macro_rules! base_dir {
        () => {
            "target/tmp"
        };
    }

    #[test]
    fn test_set_len_extend() {
        std::fs::create_dir_all(base_dir!()).unwrap();
        let path = concat!(base_dir!(), "/test_set_len_extend");

        let f = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(path)
            .unwrap();
        let mut bf = BufFile::new("tes", f).unwrap();

        let initial_size = 10;
        let extended_size = 20;
        bf.write_all(&vec![1u8; initial_size]).unwrap();
        bf.set_len(extended_size).unwrap();

        bf.rewind().unwrap();

        let mut br = Vec::new();
        bf.read_to_end(&mut br).unwrap();

        let mut expected_data = vec![1u8; initial_size];
        expected_data.extend_from_slice(&vec![0u8; extended_size as usize - initial_size]);

        assert_eq!(br, expected_data);
    }

    #[test]
    fn test_read_from_empty_file() {
        std::fs::create_dir_all(base_dir!()).unwrap();
        let path = concat!(base_dir!(), "/test_read_from_empty_file");

        let f = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(path)
            .unwrap();
        let mut bf = BufFile::new("tes", f).unwrap();

        let mut br = Vec::new();
        let result = bf.read_to_end(&mut br).unwrap();

        assert_eq!(result, 0);
        assert!(br.is_empty());
    }

    #[test]
    fn test_write_to_readonly_file() {
        std::fs::create_dir_all(base_dir!()).unwrap();
        let path = concat!(base_dir!(), "/test_write_to_readonly_file");

        {
            let f = OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(path)
                .unwrap();
            let mut bf = BufFile::new("tes", f).unwrap();
            bf.write_all(b"initial data").unwrap();
            bf.flush().unwrap();
        }

        {
            let f = OpenOptions::new().read(true).open(path).unwrap();
            let mut bf = BufFile::new("tes", f).unwrap();
            bf.write_all(b"new data").unwrap();
            let result = bf.flush();
            assert!(result.is_err());
        }
    }
}
