#[macro_use]
mod helper;

#[cfg(test)]
mod test_seek_bug {
    use function_name::named;
    use rabuf::BufFile;
    use std::io::{Seek, SeekFrom, Write};

    #[named]
    #[test]
    fn test_seek_from_end_positive() {
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::new("tes", f).unwrap();
        bf.write_all(b"data").unwrap(); // File length 4

        // Seek 5 bytes past the end (should be position 4 + 5 = 9)
        let pos = bf.seek(SeekFrom::End(5)).unwrap();
        
        // Assert position is 9 (current end 4 + 5)
        assert_eq!(pos, 9);
    }
}
