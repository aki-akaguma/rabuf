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
    fn test_set_len_smaller_and_write() {
        std::fs::create_dir_all(base_dir!()).unwrap();
        let path = concat!(base_dir!(), "/test_set_len_smaller_and_write");

        let f = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(path)
            .unwrap();
        let mut bf = BufFile::new("tes", f).unwrap();

        let initial_data = b"0123456789abcdef";
        bf.write_all(initial_data).unwrap();

        let new_len = 8;
        bf.set_len(new_len).unwrap();

        let new_data = b"ghij";
        bf.seek(std::io::SeekFrom::Start(new_len)).unwrap();
        bf.write_all(new_data).unwrap();

        bf.rewind().unwrap();

        let mut br = Vec::new();
        bf.read_to_end(&mut br).unwrap();

        let mut expected_data = initial_data[..new_len as usize].to_vec();
        expected_data.extend_from_slice(new_data);

        assert_eq!(br, expected_data);
    }

    #[test]
    fn test_prepare() {
        std::fs::create_dir_all(base_dir!()).unwrap();
        let path = concat!(base_dir!(), "/test_prepare");

        let f = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(path)
            .unwrap();
        let mut bf = BufFile::with_capacity("tes", f, 16, 4).unwrap();

        let data = vec![0u8; 100];
        bf.write_all(&data).unwrap();

        bf.clear().unwrap();

        bf.prepare(50).unwrap();

        // This should be a cache hit
        let mut br = vec![0u8; 10];
        bf.seek(std::io::SeekFrom::Start(50)).unwrap();
        bf.read_exact(&mut br).unwrap();

        assert_eq!(br, &data[50..60]);
    }

    #[test]
    fn test_file_size_is_buffer_size() {
        std::fs::create_dir_all(base_dir!()).unwrap();
        let path = concat!(base_dir!(), "/test_file_size_is_buffer_size");

        let chunk_size = 16u32;
        let num_chunks = 4u16;
        let buffer_size = chunk_size * num_chunks as u32;

        let f = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(path)
            .unwrap();
        let mut bf = BufFile::with_capacity("tes", f, chunk_size, num_chunks).unwrap();

        let data = vec![0u8; buffer_size as usize];
        bf.write_all(&data).unwrap();

        bf.rewind().unwrap();

        let mut br = Vec::new();
        bf.read_to_end(&mut br).unwrap();

        assert_eq!(br, data);
    }
}
