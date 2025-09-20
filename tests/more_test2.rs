#[cfg(test)]
mod test {
    use rabuf::{BufFile, FileSetLen, SmallRead, SmallWrite};
    use std::fs::OpenOptions;
    use std::io::{Read, Seek, Write};
    //
    macro_rules! base_dir {
        () => {
            "target/tmp"
        };
    }
    //
    #[test]
    fn test_set_len_truncate() {
        std::fs::create_dir_all(base_dir!()).unwrap();
        let path = concat!(base_dir!(), "/test_set_len_truncate");
        //
        let bw = b"ABCEDFG\nhijklmn\n";
        let f = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(path)
            .unwrap();
        let mut bf = BufFile::new("tes", f).unwrap();
        bf.write_all(bw).unwrap();

        // Truncate the file
        let new_len = 5;
        bf.set_len(new_len).unwrap();

        bf.rewind().unwrap();

        let mut br = Vec::new();
        bf.read_to_end(&mut br).unwrap();
        assert_eq!(br.len() as u64, new_len);
        assert_eq!(&br, &bw[..new_len as usize]);
    }

    #[test]
    fn test_small_read_write() {
        std::fs::create_dir_all(base_dir!()).unwrap();
        let path = concat!(base_dir!(), "/test_small_read_write");
        //
        let f = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(path)
            .unwrap();
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

    #[test]
    fn test_read_fill_buffer() {
        std::fs::create_dir_all(base_dir!()).unwrap();
        let path = concat!(base_dir!(), "/test_read_fill_buffer");
        //
        let bw = b"This is a test file for read_fill_buffer.";
        let f = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(path)
            .unwrap();
        let mut bf = BufFile::with_capacity("tes", f, 16, 4).unwrap();
        bf.write_all(bw).unwrap();

        bf.clear().unwrap();
        bf.read_fill_buffer().unwrap();

        bf.rewind().unwrap();
        let mut br = vec![0u8; bw.len()];
        bf.read_exact(&mut br).unwrap();
        assert_eq!(&br, bw);
    }

    #[test]
    fn test_write_zero() {
        std::fs::create_dir_all(base_dir!()).unwrap();
        let path = concat!(base_dir!(), "/test_write_zero");
        //
        let f = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(path)
            .unwrap();
        let mut bf = BufFile::new("tes", f).unwrap();

        let size_to_write = 10;
        bf.write_zero(size_to_write).unwrap();

        bf.rewind().unwrap();

        let mut br = vec![0u8; size_to_write as usize];
        bf.read_exact(&mut br).unwrap();
        assert_eq!(br, vec![0u8; size_to_write as usize]);
    }
}
