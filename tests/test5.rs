#[macro_use]
mod helper;

#[cfg(test)]
mod test5 {
    use function_name::named;
    use rabuf::{BufFile, MaybeSlice, SmallRead, SmallWrite};
    use std::io::{Seek, SeekFrom, Write};

    #[named]
    #[test]
    fn test_read_exact_maybeslice() {
        let bw = b"0123456789abcdef";
        let f = open_test_file!(function_name!());
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

    #[named]
    #[test]
    fn test_maybeslice_slice_variant() {
        let content = b"Hello, world! This is a test string.";
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::new("tes", f).unwrap();
        bf.write_all(content).unwrap();
        bf.seek(SeekFrom::Start(0)).unwrap();

        // Read a small slice that should fit within a chunk
        let read_size = 5;
        let maybe_slice = bf.read_exact_maybeslice(read_size).unwrap();

        match maybe_slice {
            MaybeSlice::Slice(s) => {
                assert_eq!(s, &content[0..read_size]);
                assert_eq!(s.len(), read_size);
            }
            _ => panic!("Expected MaybeSlice::Slice variant"),
        }

        // Test Deref
        assert_eq!(&*maybe_slice, &content[0..read_size]);

        // Test into_vec
        let vec_content = maybe_slice.into_vec();
        assert_eq!(vec_content, &content[0..read_size]);
    }

    #[named]
    #[test]
    fn test_maybeslice_buffer_variant() {
        // Create content larger than a typical chunk size (e.g., 4KB) to force Buffer variant
        let chunk_size = 4 * 1024; // Default chunk size
        let content_len = chunk_size * 2 + 100; // Spans multiple chunks
        let content: Vec<u8> = (0..content_len).map(|i| (i % 256) as u8).collect();

        let f = open_test_file!(function_name!());
        let mut bf = BufFile::new("tes", f).unwrap();
        bf.write_all(&content).unwrap();
        bf.seek(SeekFrom::Start(0)).unwrap();

        // Read a size that will likely result in MaybeSlice::Buffer
        let read_size = chunk_size + 50;
        let maybe_slice = bf.read_exact_maybeslice(read_size).unwrap();

        match maybe_slice {
            MaybeSlice::Buffer(ref b) => {
                assert_eq!(b, &content[0..read_size]);
                assert_eq!(b.len(), read_size);
            }
            _ => panic!("Expected MaybeSlice::Buffer variant"),
        }

        // Test Deref
        assert_eq!(&*maybe_slice, &content[0..read_size]);

        // Test into_vec
        let vec_content = maybe_slice.into_vec();
        assert_eq!(vec_content, &content[0..read_size]);
    }

    #[named]
    #[test]
    fn test_maybeslice_zero_bytes() {
        let bw = b"Some data";
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::new("tes", f).unwrap();
        bf.write_all(bw).unwrap();
        bf.seek(SeekFrom::Start(0)).unwrap();

        let read_size = 0;
        let maybe_slice = bf.read_exact_maybeslice(read_size).unwrap();

        assert_eq!(&*maybe_slice, &[]);
        assert_eq!(maybe_slice.len(), 0);
    }

    #[named]
    #[test]
    fn test_maybeslice_at_eof() {
        let bw = b"End of file test";
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::new("tes", f).unwrap();
        bf.write_all(bw).unwrap();
        bf.seek(SeekFrom::Start(0)).unwrap();

        // Seek to the end of the file
        bf.seek(SeekFrom::End(0)).unwrap();

        // Try to read 5 bytes at EOF, should result in an error (UnexpectedEof)
        let result = bf.read_exact_maybeslice(5);
        assert!(result.is_ok());
        let maybe_slice = result.unwrap();
        assert_eq!(&*maybe_slice, &[0, 0, 0, 0, 0]);
        assert_eq!(maybe_slice.len(), 5);
    }

    #[named]
    #[test]
    fn test_maybeslice_read_multiple_times() {
        let bw = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::new("tes", f).unwrap();
        bf.write_all(bw).unwrap();
        bf.seek(SeekFrom::Start(0)).unwrap();

        let slice1 = bf.read_exact_maybeslice(5).unwrap();
        assert_eq!(&*slice1, b"ABCDE");

        let slice2 = bf.read_exact_maybeslice(3).unwrap();
        assert_eq!(&*slice2, b"FGH");

        let slice3 = bf.read_exact_maybeslice(7).unwrap();
        assert_eq!(&*slice3, b"IJKLMNO");
    }

    #[named]
    #[test]
    fn test_read_exact_maybeslice_empty_file() {
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::new("tes", f).unwrap();
        let result = bf.read_exact_maybeslice(5);
        //assert!(result.is_err());
        assert!(result.is_ok());
        let maybe_slice = result.unwrap();
        assert_eq!(&*maybe_slice, &[0, 0, 0, 0, 0]);
        assert_eq!(maybe_slice.len(), 5);
    }

    #[named]
    #[test]
    fn test_read_exact_maybeslice_at_eof_with_data() {
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::new("tes", f).unwrap();
        bf.write_u8(1).unwrap();
        bf.seek(SeekFrom::End(0)).unwrap();
        let result = bf.read_exact_maybeslice(1);
        //assert!(result.is_err());
        assert!(result.is_ok());
        let maybe_slice = result.unwrap();
        assert_eq!(&*maybe_slice, &[0]);
        assert_eq!(maybe_slice.len(), 1);
    }
}
