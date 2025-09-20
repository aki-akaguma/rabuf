#[cfg(test)]
mod test {
    use rabuf::{BufFile, MaybeSlice, SmallRead};
    use std::fs::OpenOptions;
    use std::io::{Read, Seek, SeekFrom, Write};

    macro_rules! base_dir {
        () => {
            "target/tmp"
        };
    }

    #[test]
    fn test_seek_and_read() {
        std::fs::create_dir_all(base_dir!()).unwrap();
        let path = concat!(base_dir!(), "/test_seek_and_read");

        let bw = b"0123456789abcdefghijklmnopqrstuvwxyz";
        let f = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(path)
            .unwrap();
        let mut bf = BufFile::with_capacity("tes", f, 8, 4).unwrap();
        bf.write_all(bw).unwrap();

        // Seek to the middle and read
        let seek_pos = 10;
        bf.seek(SeekFrom::Start(seek_pos)).unwrap();
        let mut br = vec![0u8; 5];
        bf.read_exact(&mut br).unwrap();
        assert_eq!(&br, &bw[seek_pos as usize..(seek_pos + 5) as usize]);

        // Seek from current position and read
        bf.seek(SeekFrom::Current(5)).unwrap();
        let mut br = vec![0u8; 5];
        bf.read_exact(&mut br).unwrap();
        assert_eq!(
            &br,
            &bw[(seek_pos + 5 + 5) as usize..(seek_pos + 5 + 5 + 5) as usize]
        );

        // Seek from end and read
        let seek_from_end = 10;
        bf.seek(SeekFrom::End(-(seek_from_end as i64))).unwrap();
        let mut br = vec![0u8; 5];
        bf.read_exact(&mut br).unwrap();
        assert_eq!(
            &br,
            &bw[(bw.len() - seek_from_end)..(bw.len() - seek_from_end + 5)]
        );
    }

    #[test]
    fn test_chunk_boundary() {
        std::fs::create_dir_all(base_dir!()).unwrap();
        let path = concat!(base_dir!(), "/test_chunk_boundary");

        let chunk_size = 8;
        let bw = b"0123456789abcdef";
        let f = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(path)
            .unwrap();
        let mut bf = BufFile::with_capacity("tes", f, chunk_size, 2).unwrap();
        bf.write_all(bw).unwrap();

        // Read across chunk boundary
        bf.seek(SeekFrom::Start(chunk_size as u64 - 4)).unwrap();
        let mut br = vec![0u8; 8];
        bf.read_exact(&mut br).unwrap();
        assert_eq!(
            &br,
            &bw[(chunk_size - 4) as usize..(chunk_size + 4) as usize]
        );

        // Write across chunk boundary
        bf.seek(SeekFrom::Start(chunk_size as u64 - 4)).unwrap();
        let new_data = b"testdata";
        bf.write_all(new_data).unwrap();

        bf.seek(SeekFrom::Start(0)).unwrap();
        let mut br = vec![0u8; bw.len()];
        bf.read_exact(&mut br).unwrap();

        let mut expected_data = bw.to_vec();
        expected_data.splice(
            (chunk_size - 4) as usize..(chunk_size + 4) as usize,
            new_data.iter().cloned(),
        );
        assert_eq!(&br, &expected_data);
    }

    #[test]
    fn test_read_exact_maybeslice() {
        std::fs::create_dir_all(base_dir!()).unwrap();
        let path = concat!(base_dir!(), "/test_read_exact_maybeslice");

        let bw = b"0123456789abcdef";
        let f = OpenOptions::new()
            .create(true)
            .truncate(true)
            .read(true)
            .write(true)
            .open(path)
            .unwrap();
        let mut bf = BufFile::with_capacity("tes", f, 8, 2).unwrap();
        bf.write_all(bw).unwrap();

        bf.seek(SeekFrom::Start(6)).unwrap();

        // Read across chunks
        match bf.read_exact_maybeslice(4).unwrap() {
            MaybeSlice::Slice(_) => panic!("Expected a buffer"),
            MaybeSlice::Buffer(b) => assert_eq!(b, &bw[6..10]),
        }

        // Read within a chunk
        match bf.read_exact_maybeslice(4).unwrap() {
            MaybeSlice::Slice(s) => assert_eq!(s, &bw[10..14]),
            MaybeSlice::Buffer(_) => panic!("Expected a slice"),
        }
    }
}
