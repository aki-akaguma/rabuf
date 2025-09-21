#[macro_use]
mod helper;

#[cfg(test)]
mod test4 {
    use function_name::named;
    use rabuf::{BufFile, SmallRead, SmallWrite};
    use std::io::{Seek, SeekFrom};

    #[named]
    #[test]
    fn test_small_read_write_1() {
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::new("tes", f).unwrap();

        bf.write_u8(0x01).unwrap();
        bf.write_u16_le(0x0203).unwrap();
        bf.write_u32_le(0x04050607).unwrap();
        bf.write_u64_le(0x08090a0b0c0d0e0f).unwrap();

        bf.rewind().unwrap();

        assert_eq!(bf.read_u8().unwrap(), 0x01);
        assert_eq!(bf.read_u16_le().unwrap(), 0x0203);
        assert_eq!(bf.read_u32_le().unwrap(), 0x04050607);
        assert_eq!(bf.read_u64_le().unwrap(), 0x08090a0b0c0d0e0f);
    }

    #[named]
    #[test]
    fn test_small_read_write_2() {
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::new("test_small_read_write", f).unwrap();

        // Test u8
        bf.write_u8(42).unwrap();
        bf.rewind().unwrap();
        assert_eq!(bf.read_u8().unwrap(), 42);

        // Test u16
        bf.rewind().unwrap();
        bf.write_u16_le(0x1234).unwrap();
        bf.rewind().unwrap();
        assert_eq!(bf.read_u16_le().unwrap(), 0x1234);

        // Test u32
        bf.rewind().unwrap();
        bf.write_u32_le(0x12345678).unwrap();
        bf.rewind().unwrap();
        assert_eq!(bf.read_u32_le().unwrap(), 0x12345678);

        // Test u64
        bf.rewind().unwrap();
        bf.write_u64_le(0x123456789abcdef0).unwrap();
        bf.rewind().unwrap();
        assert_eq!(bf.read_u64_le().unwrap(), 0x123456789abcdef0);

        // Test a mix
        bf.rewind().unwrap();
        bf.write_u8(1).unwrap();
        bf.write_u16_le(2).unwrap();
        bf.write_u32_le(3).unwrap();
        bf.write_u64_le(4).unwrap();

        bf.rewind().unwrap();
        assert_eq!(bf.read_u8().unwrap(), 1);
        assert_eq!(bf.read_u16_le().unwrap(), 2);
        assert_eq!(bf.read_u32_le().unwrap(), 3);
        assert_eq!(bf.read_u64_le().unwrap(), 4);
    }

    #[named]
    #[test]
    fn test_write_u8_read_u8_at_eof() {
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::new("tes", f).unwrap();
        bf.write_u8(1).unwrap();
        bf.seek(SeekFrom::End(0)).unwrap();
        let result = bf.read_u8();
        //assert!(result.is_err());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[named]
    #[test]
    fn test_write_u16_le_read_u16_le_at_eof() {
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::new("tes", f).unwrap();
        bf.write_u16_le(1).unwrap();
        bf.seek(SeekFrom::End(0)).unwrap();
        let result = bf.read_u16_le();
        //assert!(result.is_err());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[named]
    #[test]
    fn test_write_u32_le_read_u32_le_at_eof() {
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::new("tes", f).unwrap();
        bf.write_u32_le(1).unwrap();
        bf.seek(SeekFrom::End(0)).unwrap();
        let result = bf.read_u32_le();
        //assert!(result.is_err());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[named]
    #[test]
    fn test_write_u64_le_read_u64_le_at_eof() {
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::new("tes", f).unwrap();
        bf.write_u64_le(1).unwrap();
        bf.seek(SeekFrom::End(0)).unwrap();
        let result = bf.read_u64_le();
        //assert!(result.is_err());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[named]
    #[test]
    fn test_write_u8_read_max_8_bytes_at_eof() {
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::new("tes", f).unwrap();
        bf.write_u8(1).unwrap();
        bf.seek(SeekFrom::End(0)).unwrap();
        let result = bf.read_max_8_bytes(1);
        //assert!(result.is_err());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[named]
    #[test]
    fn test_write_u8_read_exact_small_at_eof() {
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::new("tes", f).unwrap();
        bf.write_u8(1).unwrap();
        bf.seek(SeekFrom::End(0)).unwrap();
        let mut buf = [0u8; 1];
        let result = bf.read_exact_small(&mut buf);
        //assert!(result.is_err());
        assert!(result.is_ok());
        assert_eq!(&buf, &[0]);
    }
}
