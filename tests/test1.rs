#[macro_use]
mod helper;

#[cfg(test)]
mod test1 {
    use function_name::named;
    use rabuf::BufFile;
    use std::io::{Read, Seek, SeekFrom, Write};

    #[named]
    #[test]
    fn test_rewind() {
        let bw = b"ABCEDFG\nhijklmn\n";
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::new("tes", f).unwrap();
        bf.write_all(bw).unwrap();
        //
        bf.rewind().unwrap();
        //
        let mut br = vec![0u8; bw.len()];
        bf.read_exact(&mut br).unwrap();
        assert_eq!(&br, bw);
    }

    #[named]
    #[test]
    fn test_close_and_open() {
        let bw = b"abcdefg\nHIJKLMN\n";
        {
            let f = open_test_file!(function_name!());
            let mut bf = BufFile::new("tes", f).unwrap();
            bf.write_all(bw).unwrap();
        }
        {
            let path = concat!(base_dir!(), "/", function_name!());
            let f = std::fs::OpenOptions::new()
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

    #[named]
    #[test]
    fn test_with_capacity() {
        let bw = b"1234567\nABCDEFG\n8901234\nabcdefg\n";
        {
            let f = open_test_file!(function_name!());
            let mut bf = BufFile::with_capacity("tes", f, 2, 4).unwrap();
            bf.write_all(bw).unwrap();
        }
        {
            let path = concat!(base_dir!(), "/", function_name!());
            let f = std::fs::OpenOptions::new()
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

    #[named]
    #[test]
    fn test_seek_over_the_end() {
        let bw = b"abcdefg\n";
        let pos = {
            let f = open_test_file!(function_name!());
            let mut bf = BufFile::with_capacity("tes", f, 2, 4).unwrap();
            bf.seek(SeekFrom::End(0)).unwrap();
            // test a sparse file
            let pos = bf.seek(SeekFrom::Current(16)).unwrap();
            bf.write_all(bw).unwrap();
            pos
        };
        {
            let path = concat!(base_dir!(), "/", function_name!());
            let f = std::fs::OpenOptions::new()
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

    #[named]
    #[test]
    fn test_read_fill_buffer_empty_file() {
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::new("tes", f).unwrap();

        bf.read_fill_buffer().unwrap();

        let mut br = Vec::new();
        let result = bf.read_to_end(&mut br).unwrap();

        assert_eq!(result, 0);
        assert!(br.is_empty());
    }

    #[named]
    #[test]
    fn test_read_fill_buffer_small_file() {
        let f = open_test_file!(function_name!());
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

    #[named]
    #[test]
    fn test_seek_negative_from_current() {
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::new("tes", f).unwrap();

        let data = b"0123456789";
        bf.write_all(data).unwrap();

        bf.seek(SeekFrom::Start(8)).unwrap();
        bf.seek(SeekFrom::Current(-4)).unwrap();

        let mut br = vec![0u8; 4];
        bf.read_exact(&mut br).unwrap();

        assert_eq!(br, &data[4..8]);
    }

    #[named]
    #[test]
    fn test_small_chunk_size() {
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::with_capacity("tes", f, 2, 8).unwrap();

        let data = b"This is a test with a small chunk size.";
        bf.write_all(data).unwrap();

        bf.rewind().unwrap();

        let mut br = Vec::new();
        bf.read_to_end(&mut br).unwrap();

        assert_eq!(br, data);
    }

    #[named]
    #[test]
    fn test_file_not_multiple_of_chunk_size() {
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::with_capacity("tes", f, 16, 4).unwrap();

        let data = vec![0u8; 50]; // Not a multiple of 16
        bf.write_all(&data).unwrap();

        bf.rewind().unwrap();

        let mut br = Vec::new();
        bf.read_to_end(&mut br).unwrap();

        assert_eq!(br, data);
    }

    #[named]
    #[test]
    fn test_clear() {
        let bw = b"Data to be cleared";
        {
            let f = open_test_file!(function_name!());
            let mut bf = BufFile::new("tes", f).unwrap();
            bf.write_all(bw).unwrap();
            bf.clear().unwrap();
        }

        {
            let path = concat!(base_dir!(), "/", function_name!());
            let mut f = std::fs::File::open(path).unwrap();
            let mut br = Vec::new();
            f.read_to_end(&mut br).unwrap();
            assert_eq!(&br, bw);
        }
    }

    #[named]
    #[test]
    fn test_flush() {
        let bw = b"Data to be flushed";
        {
            let f = open_test_file!(function_name!());
            let mut bf = BufFile::new("tes", f).unwrap();
            bf.write_all(bw).unwrap();
            bf.flush().unwrap();
        }

        {
            let path = concat!(base_dir!(), "/", function_name!());
            let mut f = std::fs::File::open(path).unwrap();
            let mut br = vec![0u8; bw.len()];
            f.read_exact(&mut br).unwrap();
            assert_eq!(&br, bw);
        }
    }

    #[named]
    #[test]
    fn test_read_fill_buffer() {
        let bw = b"This is a test file for read_fill_buffer.";
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::with_capacity("tes", f, 16, 4).unwrap();
        bf.write_all(bw).unwrap();

        bf.clear().unwrap();
        bf.read_fill_buffer().unwrap();

        bf.rewind().unwrap();
        let mut br = vec![0u8; bw.len()];
        bf.read_exact(&mut br).unwrap();
        assert_eq!(&br, bw);
    }

    #[named]
    #[test]
    fn test_read_from_empty_file() {
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::new("tes", f).unwrap();

        let mut br = Vec::new();
        let result = bf.read_to_end(&mut br).unwrap();

        assert_eq!(result, 0);
        assert!(br.is_empty());
    }

    #[named]
    #[test]
    fn test_prepare() {
        let f = open_test_file!(function_name!());
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

    #[named]
    #[test]
    fn test_file_size_is_buffer_size() {
        let f = open_test_file!(function_name!());

        let chunk_size = 16u32;
        let num_chunks = 4u16;
        let buffer_size = chunk_size * num_chunks as u32;

        let mut bf = BufFile::with_capacity("tes", f, chunk_size, num_chunks).unwrap();

        let data = vec![0u8; buffer_size as usize];
        bf.write_all(&data).unwrap();

        bf.rewind().unwrap();

        let mut br = Vec::new();
        bf.read_to_end(&mut br).unwrap();

        assert_eq!(br, data);
    }

    #[named]
    #[test]
    fn test_write_to_readonly_file() {
        {
            let f = open_test_file!(function_name!());
            let mut bf = BufFile::new("tes", f).unwrap();
            bf.write_all(b"initial data").unwrap();
            bf.flush().unwrap();
        }
        {
            let path = concat!(base_dir!(), "/", function_name!());
            let f = std::fs::OpenOptions::new().read(true).open(path).unwrap();
            let mut bf = BufFile::new("tes", f).unwrap();
            bf.write_all(b"new data").unwrap();
            let result = bf.flush();
            assert!(result.is_err());
        }
    }

    #[named]
    #[test]
    fn test_seek_and_read() {
        let bw = b"0123456789abcdefghijklmnopqrstuvwxyz";
        let f = open_test_file!(function_name!());
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

    #[named]
    #[test]
    fn test_chunk_boundary() {
        let bw = b"0123456789abcdef";
        let chunk_size = 8;
        let f = open_test_file!(function_name!());
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

    #[named]
    #[test]
    fn test_large_data() {
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::with_capacity("tes", f, 1024, 4).unwrap();

        let data_size = 8192;
        let mut data = Vec::with_capacity(data_size);
        for i in 0..data_size {
            data.push((i % 256) as u8);
        }

        bf.write_all(&data).unwrap();

        bf.rewind().unwrap();

        let mut br = Vec::new();
        bf.read_to_end(&mut br).unwrap();

        assert_eq!(br, data);
    }
}
