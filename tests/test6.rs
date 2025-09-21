#[macro_use]
mod helper;

#[cfg(test)]
mod test6 {
    use function_name::named;
    use rabuf::{BufFile, MaybeSlice, SmallRead, SmallWrite};
    use std::io::{Read, Seek, SeekFrom, Write};

    const CHUNK_SIZE: u32 = 1024 * 4;

    #[named]
    #[test]
    fn test_read_exact_small_chunk_boundary() {
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::with_capacity("tes", f, CHUNK_SIZE, 2).unwrap();
        let data = vec![0xAA; (CHUNK_SIZE * 2) as usize];
        bf.write_all(&data).unwrap();

        bf.seek(SeekFrom::Start((CHUNK_SIZE - 5) as u64)).unwrap();
        let mut buf = [0u8; 10];
        bf.read_exact_small(&mut buf).unwrap();
        assert_eq!(buf, [0xAA; 10]);
    }

    #[named]
    #[test]
    fn test_read_exact_maybeslice_chunk_boundary() {
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::with_capacity("tes", f, CHUNK_SIZE, 2).unwrap();
        let data = vec![0xBB; (CHUNK_SIZE * 2) as usize];
        bf.write_all(&data).unwrap();

        bf.seek(SeekFrom::Start((CHUNK_SIZE - 5) as u64)).unwrap();
        let maybe_slice = bf.read_exact_maybeslice(10).unwrap();
        assert_eq!(&*maybe_slice, &[0xBB; 10]);
    }

    #[named]
    #[test]
    fn test_read_exact_maybeslice_larger_than_chunk() {
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::with_capacity("tes", f, CHUNK_SIZE, 2).unwrap();
        let data = vec![0xCC; (CHUNK_SIZE * 2) as usize];
        bf.write_all(&data).unwrap();

        bf.rewind().unwrap();
        let maybe_slice = bf
            .read_exact_maybeslice((CHUNK_SIZE + 10) as usize)
            .unwrap();
        match maybe_slice {
            MaybeSlice::Buffer(b) => assert_eq!(b, vec![0xCC; (CHUNK_SIZE + 10) as usize]),
            _ => panic!("Expected MaybeSlice::Buffer"),
        }
    }

    #[named]
    #[test]
    fn test_write_u8_chunk_boundaries() {
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::with_capacity("tes", f, CHUNK_SIZE, 2).unwrap();

        // Write at the beginning of a chunk
        bf.seek(SeekFrom::Start(CHUNK_SIZE as u64)).unwrap();
        bf.write_u8(0xA1).unwrap();
        bf.seek(SeekFrom::Start(CHUNK_SIZE as u64)).unwrap();
        assert_eq!(bf.read_u8().unwrap(), 0xA1);

        // Write at the end of a chunk
        bf.seek(SeekFrom::Start(CHUNK_SIZE as u64 - 1)).unwrap();
        bf.write_u8(0xB2).unwrap();
        bf.seek(SeekFrom::Start(CHUNK_SIZE as u64 - 1)).unwrap();
        assert_eq!(bf.read_u8().unwrap(), 0xB2);

        // Write across a chunk boundary (last byte of first chunk, first byte of second chunk)
        bf.seek(SeekFrom::Start(CHUNK_SIZE as u64 - 1)).unwrap();
        bf.write_u8(0xC3).unwrap(); // This will write to the end of the first chunk
        bf.write_u8(0xD4).unwrap(); // This will write to the beginning of the second chunk
        bf.seek(SeekFrom::Start(CHUNK_SIZE as u64 - 1)).unwrap();
        assert_eq!(bf.read_u8().unwrap(), 0xC3);
        assert_eq!(bf.read_u8().unwrap(), 0xD4);
    }

    #[named]
    #[test]
    fn test_write_u16_le_chunk_boundaries() {
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::with_capacity("tes", f, CHUNK_SIZE, 2).unwrap();

        // Write at the beginning of a chunk
        bf.seek(SeekFrom::Start(CHUNK_SIZE as u64)).unwrap();
        bf.write_u16_le(0xA1B2).unwrap();
        bf.seek(SeekFrom::Start(CHUNK_SIZE as u64)).unwrap();
        assert_eq!(bf.read_u16_le().unwrap(), 0xA1B2);

        // Write at the end of a chunk (last 2 bytes)
        bf.seek(SeekFrom::Start(CHUNK_SIZE as u64 - 2)).unwrap();
        bf.write_u16_le(0xC3D4).unwrap();
        bf.seek(SeekFrom::Start(CHUNK_SIZE as u64 - 2)).unwrap();
        assert_eq!(bf.read_u16_le().unwrap(), 0xC3D4);

        // Write across a chunk boundary (last byte of first chunk, first byte of second chunk)
        bf.seek(SeekFrom::Start(CHUNK_SIZE as u64 - 1)).unwrap();
        bf.write_u16_le(0xE5F6).unwrap(); // This will write 1 byte to first chunk, 1 byte to second
        bf.seek(SeekFrom::Start(CHUNK_SIZE as u64 - 1)).unwrap();
        assert_eq!(bf.read_u16_le().unwrap(), 0xE5F6);
    }

    #[named]
    #[test]
    fn test_write_u32_le_chunk_boundaries() {
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::with_capacity("tes", f, CHUNK_SIZE, 2).unwrap();

        // Write at the beginning of a chunk
        bf.seek(SeekFrom::Start(CHUNK_SIZE as u64)).unwrap();
        bf.write_u32_le(0xA1B2C3D4).unwrap();
        bf.seek(SeekFrom::Start(CHUNK_SIZE as u64)).unwrap();
        assert_eq!(bf.read_u32_le().unwrap(), 0xA1B2C3D4);

        // Write at the end of a chunk (last 4 bytes)
        bf.seek(SeekFrom::Start(CHUNK_SIZE as u64 - 4)).unwrap();
        bf.write_u32_le(0xE5F6A7B8).unwrap();
        bf.seek(SeekFrom::Start(CHUNK_SIZE as u64 - 4)).unwrap();
        assert_eq!(bf.read_u32_le().unwrap(), 0xE5F6A7B8);

        // Write across a chunk boundary (e.g., 2 bytes in first chunk, 2 bytes in second)
        bf.seek(SeekFrom::Start(CHUNK_SIZE as u64 - 2)).unwrap();
        bf.write_u32_le(0xC1C2C3C4).unwrap();
        bf.seek(SeekFrom::Start(CHUNK_SIZE as u64 - 2)).unwrap();
        assert_eq!(bf.read_u32_le().unwrap(), 0xC1C2C3C4);
    }

    #[named]
    #[test]
    fn test_write_u64_le_chunk_boundaries() {
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::with_capacity("tes", f, CHUNK_SIZE, 2).unwrap();

        // Write at the beginning of a chunk
        bf.seek(SeekFrom::Start(CHUNK_SIZE as u64)).unwrap();
        bf.write_u64_le(0x0102030405060708).unwrap();
        bf.seek(SeekFrom::Start(CHUNK_SIZE as u64)).unwrap();
        assert_eq!(bf.read_u64_le().unwrap(), 0x0102030405060708);

        // Write at the end of a chunk (last 8 bytes)
        bf.seek(SeekFrom::Start(CHUNK_SIZE as u64 - 8)).unwrap();
        bf.write_u64_le(0x090A0B0C0D0E0F10).unwrap();
        bf.seek(SeekFrom::Start(CHUNK_SIZE as u64 - 8)).unwrap();
        assert_eq!(bf.read_u64_le().unwrap(), 0x090A0B0C0D0E0F10);

        // Write across a chunk boundary (e.g., 4 bytes in first chunk, 4 bytes in second)
        bf.seek(SeekFrom::Start(CHUNK_SIZE as u64 - 4)).unwrap();
        bf.write_u64_le(0x1112131415161718).unwrap();
        bf.seek(SeekFrom::Start(CHUNK_SIZE as u64 - 4)).unwrap();
        assert_eq!(bf.read_u64_le().unwrap(), 0x1112131415161718);
    }

    #[named]
    #[test]
    fn test_write_u64_le_slice_empty() {
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::new("tes", f).unwrap();
        let data: Vec<u64> = vec![];
        bf.write_u64_le_slice(&data).unwrap();
        bf.rewind().unwrap();
        let mut br = Vec::new();
        bf.read_to_end(&mut br).unwrap();
        assert!(br.is_empty());
    }

    #[named]
    #[test]
    fn test_write_u64_le_slice_chunk_boundary() {
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::with_capacity("tes", f, CHUNK_SIZE, 2).unwrap();

        let val1 = 0x1111111111111111u64;
        let val2 = 0x2222222222222222u64;
        let val3 = 0x3333333333333333u64;

        // Position just before chunk boundary, write 3 u64s (24 bytes)
        // This will cross the chunk boundary if CHUNK_SIZE is not a multiple of 8
        // or if the position is not aligned.
        let start_pos = CHUNK_SIZE as u64 - 8; // Write 1 u64 in first chunk, 2 in second
        bf.seek(SeekFrom::Start(start_pos)).unwrap();
        bf.write_u64_le_slice(&[val1, val2, val3]).unwrap();

        bf.seek(SeekFrom::Start(start_pos)).unwrap();
        assert_eq!(bf.read_u64_le().unwrap(), val1);
        assert_eq!(bf.read_u64_le().unwrap(), val2);
        assert_eq!(bf.read_u64_le().unwrap(), val3);
    }

    #[named]
    #[test]
    fn test_write_u64_le_slice2_empty() {
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::new("tes", f).unwrap();
        let data1: Vec<u64> = vec![];
        let data2: Vec<u64> = vec![];
        bf.write_u64_le_slice2(&data1, &data2).unwrap();
        bf.rewind().unwrap();
        let mut br = Vec::new();
        bf.read_to_end(&mut br).unwrap();
        assert!(br.is_empty());
    }

    #[named]
    #[test]
    fn test_write_u64_le_slice2_chunk_boundary() {
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::with_capacity("tes", f, CHUNK_SIZE, 2).unwrap();

        let val1 = 0x1111111111111111u64;
        let val2 = 0x2222222222222222u64;
        let val3 = 0x3333333333333333u64;
        let val4 = 0x4444444444444444u64;

        // Position just before chunk boundary, write 2 u64s from slice1 and 2 from slice2
        let start_pos = CHUNK_SIZE as u64 - 8; // Write 1 u64 from slice1 in first chunk, rest in second
        bf.seek(SeekFrom::Start(start_pos)).unwrap();
        bf.write_u64_le_slice2(&[val1, val2], &[val3, val4])
            .unwrap();

        bf.seek(SeekFrom::Start(start_pos)).unwrap();
        assert_eq!(bf.read_u64_le().unwrap(), val1);
        assert_eq!(bf.read_u64_le().unwrap(), val2);
        assert_eq!(bf.read_u64_le().unwrap(), val3);
        assert_eq!(bf.read_u64_le().unwrap(), val4);
    }

    #[named]
    #[test]
    fn test_write_all_small_exact_chunk_size() {
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::with_capacity("tes", f, CHUNK_SIZE, 2).unwrap();
        let data = vec![0xEE; CHUNK_SIZE as usize];
        bf.write_all_small(&data).unwrap();
        bf.rewind().unwrap();
        let mut br = vec![0u8; CHUNK_SIZE as usize];
        bf.read_exact(&mut br).unwrap();
        assert_eq!(br, data);
    }

    #[named]
    #[test]
    fn test_write_all_small_less_than_chunk_size() {
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::with_capacity("tes", f, CHUNK_SIZE, 2).unwrap();
        let data = vec![0xFF; (CHUNK_SIZE / 2) as usize];
        bf.write_all_small(&data).unwrap();
        bf.rewind().unwrap();
        let mut br = vec![0u8; (CHUNK_SIZE / 2) as usize];
        bf.read_exact(&mut br).unwrap();
        assert_eq!(br, data);
    }

    #[named]
    #[test]
    fn test_write_all_small_chunk_boundary() {
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::with_capacity("tes", f, CHUNK_SIZE, 2).unwrap();

        let data_part1 = vec![0x11; (CHUNK_SIZE - 10) as usize];
        let data_part2 = vec![0x22; 20]; // 10 bytes in first chunk, 10 in second

        bf.write_all_small(&data_part1).unwrap();
        bf.write_all_small(&data_part2).unwrap();

        bf.rewind().unwrap();
        let mut br = vec![0u8; (CHUNK_SIZE - 10 + 20) as usize];
        bf.read_exact(&mut br).unwrap();

        let mut expected = data_part1;
        expected.extend_from_slice(&data_part2);
        assert_eq!(br, expected);
    }

    #[named]
    #[test]
    fn test_write_zero_chunk_boundary() {
        let f = open_test_file!(function_name!());
        let mut bf = BufFile::with_capacity("tes", f, CHUNK_SIZE, 2).unwrap();

        let size_to_write = CHUNK_SIZE + 10; // Crosses chunk boundary
        bf.write_zero(size_to_write).unwrap();

        bf.rewind().unwrap();
        let mut br = vec![0u8; size_to_write as usize];
        bf.read_exact(&mut br).unwrap();
        assert_eq!(br, vec![0u8; size_to_write as usize]);
    }
}
