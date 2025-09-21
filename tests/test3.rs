#[macro_use]
mod helper;

#[cfg(test)]
mod test3 {
    use function_name::named;
    use rabuf::{BufFile, SmallWrite};
    use std::io::{Read, Seek};

    #[named]
    #[test]
    fn test_write_all_small_1() {
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::new("tes", f).unwrap();

        let data = b"small test data";
        bf.write_all_small(data).unwrap();

        bf.rewind().unwrap();

        let mut br = vec![0u8; data.len()];
        bf.read_exact(&mut br).unwrap();

        assert_eq!(br, data);
    }

    #[named]
    #[test]
    fn test_write_all_small_2() {
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::new("test_write_all_small", f).unwrap();

        bf.write_all_small(b"hello").unwrap();
        bf.write_all_small(b" world").unwrap();

        bf.rewind().unwrap();
        let mut content = Vec::new();
        bf.read_to_end(&mut content).unwrap();
        assert_eq!(content, b"hello world");
    }

    #[named]
    #[test]
    fn test_write_u64_le_slice() {
        let f = open_test_file!(function_name!());
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

    #[named]
    #[test]
    fn test_write_u64_le_slice2() {
        let f = open_test_file!(function_name!());
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

    #[named]
    #[test]
    fn test_write_zero() {
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::new("tes", f).unwrap();

        let size_to_write = 10;
        bf.write_zero(size_to_write).unwrap();

        bf.rewind().unwrap();

        let mut br = vec![0u8; size_to_write as usize];
        bf.read_exact(&mut br).unwrap();
        assert_eq!(br, vec![0u8; size_to_write as usize]);
    }

    #[named]
    #[test]
    fn test_write_zero_zero_bytes() {
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::new("tes", f).unwrap();
        bf.write_zero(0).unwrap();
        bf.rewind().unwrap();
        let mut br = Vec::new();
        bf.read_to_end(&mut br).unwrap();
        assert!(br.is_empty());
    }

    #[named]
    #[test]
    fn test_write_zero_to_empty_file() {
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::new("tes", f).unwrap();
        let size_to_write = 100;
        bf.write_zero(size_to_write).unwrap();
        bf.rewind().unwrap();
        let mut br = vec![0u8; size_to_write as usize];
        bf.read_exact(&mut br).unwrap();
        assert_eq!(br, vec![0u8; size_to_write as usize]);
    }
}
