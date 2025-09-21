#[macro_use]
mod helper;

#[cfg(test)]
mod test2 {
    use function_name::named;
    use rabuf::{BufFile, SmallRead};
    use std::io::{Seek, SeekFrom, Write};

    #[named]
    #[test]
    fn test_read_max_8_bytes() {
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::new("tes", f).unwrap();

        let val = 0x0102030405060708u64;
        bf.write_all(&val.to_le_bytes()).unwrap();

        bf.rewind().unwrap();
        assert_eq!(bf.read_max_8_bytes(8).unwrap(), val);

        bf.rewind().unwrap();
        assert_eq!(bf.read_max_8_bytes(4).unwrap(), 0x05060708);
    }

    #[named]
    #[test]
    fn test_read_exact_small_1() {
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::new("tes", f).unwrap();

        let data = b"small test data";
        bf.write_all(data).unwrap();
        bf.rewind().unwrap();

        let mut br = vec![0u8; data.len()];
        bf.read_exact_small(&mut br).unwrap();
        assert_eq!(br, data);
    }

    #[named]
    #[test]
    fn test_read_exact_small_2() {
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::new("test_read_exact_small", f).unwrap();

        let data = b"hello world, this is a test";
        bf.write_all(data).unwrap();
        bf.rewind().unwrap();

        let mut small_buf = [0u8; 5];
        bf.read_exact_small(&mut small_buf).unwrap();
        assert_eq!(&small_buf, b"hello");

        let mut small_buf = [0u8; 6];
        bf.read_exact_small(&mut small_buf).unwrap();
        assert_eq!(&small_buf, b" world");
    }

    #[named]
    #[test]
    fn test_read_u8_various_positions() {
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::new("tes", f).unwrap();
        let data = (0..256).map(|i| i as u8).collect::<Vec<u8>>();
        bf.write_all(&data).unwrap();

        bf.rewind().unwrap();
        assert_eq!(bf.read_u8().unwrap(), 0); // Start
        bf.seek(SeekFrom::Start(100)).unwrap();
        assert_eq!(bf.read_u8().unwrap(), 100); // Middle
        bf.seek(SeekFrom::End(-1)).unwrap();
        assert_eq!(bf.read_u8().unwrap(), 255); // End
    }

    #[named]
    #[test]
    fn test_read_u16_le_various_positions() {
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::new("tes", f).unwrap();
        let data = (0..256).map(|i| i as u8).collect::<Vec<u8>>();
        bf.write_all(&data).unwrap();

        bf.rewind().unwrap();
        assert_eq!(bf.read_u16_le().unwrap(), u16::from_le_bytes([0, 1]));
        bf.seek(SeekFrom::Start(100)).unwrap();
        assert_eq!(bf.read_u16_le().unwrap(), u16::from_le_bytes([100, 101]));
        bf.seek(SeekFrom::End(-2)).unwrap();
        assert_eq!(bf.read_u16_le().unwrap(), u16::from_le_bytes([254, 255]));
    }

    #[named]
    #[test]
    fn test_read_u32_le_various_positions() {
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::new("tes", f).unwrap();
        let data = (0..256).map(|i| i as u8).collect::<Vec<u8>>();
        bf.write_all(&data).unwrap();

        bf.rewind().unwrap();
        assert_eq!(bf.read_u32_le().unwrap(), u32::from_le_bytes([0, 1, 2, 3]));
        bf.seek(SeekFrom::Start(100)).unwrap();
        assert_eq!(
            bf.read_u32_le().unwrap(),
            u32::from_le_bytes([100, 101, 102, 103])
        );
        bf.seek(SeekFrom::End(-4)).unwrap();
        assert_eq!(
            bf.read_u32_le().unwrap(),
            u32::from_le_bytes([252, 253, 254, 255])
        );
    }

    #[named]
    #[test]
    fn test_read_u64_le_various_positions() {
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::new("tes", f).unwrap();
        let data = (0..256).map(|i| i as u8).collect::<Vec<u8>>();
        bf.write_all(&data).unwrap();

        bf.rewind().unwrap();
        assert_eq!(
            bf.read_u64_le().unwrap(),
            u64::from_le_bytes([0, 1, 2, 3, 4, 5, 6, 7])
        );
        bf.seek(SeekFrom::Start(100)).unwrap();
        assert_eq!(
            bf.read_u64_le().unwrap(),
            u64::from_le_bytes([100, 101, 102, 103, 104, 105, 106, 107])
        );
        bf.seek(SeekFrom::End(-8)).unwrap();
        assert_eq!(
            bf.read_u64_le().unwrap(),
            u64::from_le_bytes([248, 249, 250, 251, 252, 253, 254, 255])
        );
    }

    #[named]
    #[test]
    fn test_read_max_8_bytes_various_sizes() {
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::new("tes", f).unwrap();
        let val = 0x0102030405060708u64;
        bf.write_all(&val.to_le_bytes()).unwrap();

        bf.rewind().unwrap();
        assert_eq!(bf.read_max_8_bytes(1).unwrap(), 0x08);
        bf.rewind().unwrap();
        assert_eq!(bf.read_max_8_bytes(2).unwrap(), 0x0708);
        bf.rewind().unwrap();
        assert_eq!(bf.read_max_8_bytes(8).unwrap(), val);
    }

    #[named]
    #[test]
    fn test_read_u8_empty_file() {
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::new("tes", f).unwrap();
        let result = bf.read_u8();
        //assert!(result.is_err());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[named]
    #[test]
    fn test_read_u16_le_empty_file() {
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::new("tes", f).unwrap();
        let result = bf.read_u16_le();
        //assert!(result.is_err());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[named]
    #[test]
    fn test_read_u32_le_empty_file() {
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::new("tes", f).unwrap();
        let result = bf.read_u32_le();
        //assert!(result.is_err());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[named]
    #[test]
    fn test_read_u64_le_empty_file() {
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::new("tes", f).unwrap();
        let result = bf.read_u64_le();
        //assert!(result.is_err());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[named]
    #[test]
    fn test_read_max_8_bytes_empty_file() {
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::new("tes", f).unwrap();
        let result = bf.read_max_8_bytes(8);
        //assert!(result.is_err());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[named]
    #[test]
    fn test_read_exact_small_empty_file() {
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::new("tes", f).unwrap();
        let mut buf = [0u8; 5];
        let result = bf.read_exact_small(&mut buf);
        //assert!(result.is_err());
        assert!(result.is_ok());
        assert_eq!(&buf, &[0, 0, 0, 0, 0]);
    }
}
