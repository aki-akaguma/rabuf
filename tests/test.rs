#[cfg(test)]
mod test {
    use rabuf::BufFile;
    use std::fs::OpenOptions;
    use std::io::{Read, Seek, SeekFrom, Write};
    //
    macro_rules! base_dir {
        () => {
            "target/tmp"
        };
    }
    //
    #[test]
    fn test_1() {
        std::fs::create_dir_all(base_dir!()).unwrap();
        //
        let path = concat!(base_dir!(), "/test_1");
        let bw = b"ABCEDFG\nhijklmn\n";
        //
        let f = OpenOptions::new()
            .truncate(true)
            .read(true)
            .write(true)
            .open(path)
            .unwrap();
        let mut bf = BufFile::new("tes", f).unwrap();
        bf.write_all(bw).unwrap();
        //
        bf.rewind().unwrap();
        //
        let mut br = vec![0u8; bw.len()];
        bf.read_exact(&mut br).unwrap();
        assert_eq!(&br, bw);
    }
    #[test]
    fn test_2() {
        std::fs::create_dir_all(base_dir!()).unwrap();
        let path = concat!(base_dir!(), "/test_2");
        //
        let bw = b"abcdefg\nHIJKLMN\n";
        {
            let f = OpenOptions::new()
                .truncate(true)
                .read(true)
                .write(true)
                .open(path)
                .unwrap();
            let mut bf = BufFile::new("tes", f).unwrap();
            bf.write_all(bw).unwrap();
        }
        {
            let f = OpenOptions::new()
                .read(true)
                .write(true)
                .open(path)
                .unwrap();
            let mut bf = BufFile::new("tes", f).unwrap();
            let mut br = vec![0u8; bw.len()];
            bf.read_exact(&mut br).unwrap();
            assert_eq!(&br, bw);
        }
    }
    #[test]
    fn test_3() {
        std::fs::create_dir_all(base_dir!()).unwrap();
        let path = concat!(base_dir!(), "/test_3");
        //
        let bw = b"1234567\nABCDEFG\n8901234\nabcdefg\n";
        {
            let f = OpenOptions::new()
                .truncate(true)
                .read(true)
                .write(true)
                .open(path)
                .unwrap();
            let mut bf = BufFile::with_capacity("tes", f, 2, 4).unwrap();
            bf.write_all(bw).unwrap();
        }
        {
            let f = OpenOptions::new()
                .read(true)
                .write(true)
                .open(path)
                .unwrap();
            let mut bf = BufFile::new("tes", f).unwrap();
            let mut br = vec![0u8; bw.len()];
            bf.read_exact(&mut br).unwrap();
            assert_eq!(&br, bw);
        }
    }
    #[test]
    fn test_seek_over_the_end() {
        std::fs::create_dir_all(base_dir!()).unwrap();
        let path = concat!(base_dir!(), "/test_seek_over_the_end");
        //
        let bw = b"abcdefg\n";
        let pos = {
            let f = OpenOptions::new()
                .truncate(true)
                .read(true)
                .write(true)
                .open(path)
                .unwrap();
            let mut bf = BufFile::with_capacity("tes", f, 2, 4).unwrap();
            bf.seek(SeekFrom::End(0)).unwrap();
            // test a sparse file
            let pos = bf.seek(SeekFrom::Current(16)).unwrap();
            bf.write_all(bw).unwrap();
            pos
        };
        {
            let f = OpenOptions::new()
                .read(true)
                .write(true)
                .open(path)
                .unwrap();
            let mut bf = BufFile::new("tes", f).unwrap();
            let mut br = vec![0u8; bw.len()];
            bf.seek(SeekFrom::Start(pos)).unwrap();
            bf.read_exact(&mut br).unwrap();
            assert_eq!(&br, bw);
        }
    }
}
