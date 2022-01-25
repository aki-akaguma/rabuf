/*!
The Buffer for random access file.

When you read and write a file,
this read and write in `Chunk` units and reduce IO operation.

# Examples

## Write, Seek, Read

```rust
use rabuf::BufFile;
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};

std::fs::create_dir_all("target/tmp").unwrap();

let path = "target/tmp/doc_test_1";
let bw = b"ABCEDFG\nhijklmn\n";

let f = OpenOptions::new().create(true)
    .read(true).write(true).open(path).unwrap();
let mut bf = BufFile::new("tes", f).unwrap();
bf.write_all(bw).unwrap();

bf.seek(SeekFrom::Start(0)).unwrap();

let mut br = vec![0u8; bw.len()];
bf.read_exact(&mut br).unwrap();
assert_eq!(&br, bw);
```

## Write, Close, Open, Read

```rust
use rabuf::BufFile;
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};

std::fs::create_dir_all("target/tmp").unwrap();
let path = "target/tmp/doc_test_2";

let bw = b"abcdefg\nHIJKLMN\n";
{
    let f = OpenOptions::new().create(true)
        .read(true).write(true).open(path).unwrap();
    let mut bf = BufFile::new("tes", f).unwrap();
    bf.write_all(bw).unwrap();
}
{
    let f = OpenOptions::new().create(true)
        .read(true).write(true).open(path).unwrap();
    let mut bf = BufFile::new("tes", f).unwrap();
    let mut br = vec![0u8; bw.len()];
    bf.read_exact(&mut br).unwrap();
    assert_eq!(&br, bw);
}
```
*/
use std::fs::File;
use std::io::{Read, Result, Seek, SeekFrom, Write};

#[cfg(feature = "buf_hash_turbo")]
use std::collections::HashMap;

#[cfg(feature = "buf_myhash")]
use std::hash::BuildHasherDefault;

#[cfg(feature = "buf_myhash")]
use std::hash::Hasher;

pub mod maybe;
pub use maybe::MaybeSlice;

/// Buffered File for ramdom access.
pub type BufFile = RaBuf<File>;

impl BufFile {
    pub fn read_fill_buffer(&mut self) -> Result<()> {
        let end_pos = self.seek(SeekFrom::End(0))?;
        let chunk_size = self.chunk_size as u64;
        let mut curr = 0;
        while curr < end_pos {
            let _ = self.fetch_chunk(curr)?;
            if self.chunks.len() < self.max_num_chunks {
                curr += chunk_size;
            } else {
                break;
            }
        }
        //
        Ok(())
    }
}

/// Truncates or extends the underlying file.
pub trait FileSetLen {
    /// Truncates or extends the underlying file, updating the size of this file to become size.
    fn set_len(&mut self, size: u64) -> Result<()>;
}

impl FileSetLen for BufFile {
    /// Truncates or extends the underlying file, updating the size of this file to become size.
    /// ref. [`std::io::File.set_len()`](https://doc.rust-lang.org/std/fs/struct.File.html#method.set_len)
    fn set_len(&mut self, size: u64) -> Result<()> {
        if self.end >= size {
            // shrink bunks
            for i in 0..self.chunks.len() {
                #[cfg(feature = "buf_debug")]
                let chunk = &self.chunks[i];
                #[cfg(not(feature = "buf_debug"))]
                let chunk = unsafe { self.chunks.get_unchecked(i) };
                //
                if chunk.offset + chunk.data.len() as u64 >= size {
                    // data end is over the new end
                    // nothing todo
                } else if chunk.offset >= size {
                    // chunk start is over the new end
                    self.map.remove(&chunk.offset);
                    self.fetch_cache = None;
                    #[cfg(not(feature = "buf_overf_rem_all"))]
                    {
                        self.chunks[i].uses = 0;
                    }
                }
            }
        }
        self.end = size;
        if self.end < self.pos {
            self.pos = self.end
        }
        self.file.set_len(size)?;
        //
        Ok(())
    }
}

impl Seek for BufFile {
    #[inline]
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        let new_pos = match pos {
            SeekFrom::Start(x) => x,
            SeekFrom::End(x) => {
                if x < 0 {
                    self.end - (-x) as u64
                } else {
                    // weren't automatically extended beyond the end.
                    self.end - x as u64
                }
            }
            SeekFrom::Current(x) => {
                if x < 0 {
                    self.pos - (-x) as u64
                } else {
                    self.pos + x as u64
                }
            }
        };
        if new_pos > self.end {
            // makes a sparse file.
            self.set_len(new_pos)?;
        }
        self.pos = new_pos;
        Ok(new_pos)
    }
}

/// File syncronization include OS-internal metadata to disk.
pub trait FileSync {
    /// Attempts to sync all OS-internal metadata to disk.
    fn sync_all(&mut self) -> Result<()>;
    /// This function is similar to sync_all, except that it might not synchronize file metadata to the filesystem.
    fn sync_data(&mut self) -> Result<()>;
}

impl FileSync for BufFile {
    /// Flush buffer and call
    /// [`std::io::File.sync_all()`](https://doc.rust-lang.org/std/fs/struct.File.html#method.sync_all)
    #[inline]
    fn sync_all(&mut self) -> Result<()> {
        self.flush()?;
        self.file.sync_all()
    }
    /// Flush buffer and call
    /// [`std::io::File.sync_data()`](https://doc.rust-lang.org/std/fs/struct.File.html#method.sync_data)
    #[inline]
    fn sync_data(&mut self) -> Result<()> {
        self.flush()?;
        self.file.sync_data()
    }
}

/// Read small bytes less than chunk size.
pub trait SmallRead {
    /// Read one byte with a fast routine.
    fn read_u8(&mut self) -> Result<u8>;
    /// Read 2 bytes with a fast routine and return the little endian u16.
    fn read_u16_le(&mut self) -> Result<u16>;
    /// Read 4 bytes with a fast routine and return the little endian u32.
    fn read_u32_le(&mut self) -> Result<u32>;
    /// Read 8 bytes with a fast routine and return the little endian u64.
    fn read_u64_le(&mut self) -> Result<u64>;
    /// Read maximum 8 bytes with a fast routine and return the little endian u64.
    fn read_max_8_bytes(&mut self, size: usize) -> Result<u64>;
    /// Read small size bytes with a fast routine. The small size is less than chunk size.
    fn read_exact_small(&mut self, buf: &mut [u8]) -> Result<()>;
    /// Read small size bytes and return MaybeSlice.
    fn read_exact_maybeslice(&mut self, size: usize) -> Result<MaybeSlice<'_>>;
}

impl SmallRead for BufFile {
    /// Read one byte with a fast routine.
    #[inline]
    fn read_u8(&mut self) -> Result<u8> {
        let curr = self.pos;
        let chunk = self.fetch_chunk(curr)?;
        let st = (curr - chunk.offset) as usize;
        if st < chunk.data.len() {
            #[cfg(feature = "buf_debug")]
            let val = chunk.data[st];
            #[cfg(not(feature = "buf_debug"))]
            let val = unsafe { *chunk.data.get_unchecked(st) };
            //
            self.pos += 1;
            Ok(val)
        } else {
            let mut buf = [0u8; 1];
            let _ = self.read_exact(&mut buf)?;
            Ok(buf[0])
        }
    }
    #[inline]
    fn read_u16_le(&mut self) -> Result<u16> {
        const SIZE: usize = 2;
        let curr = self.pos;
        let chunk = self.fetch_chunk(curr)?;
        let st = (curr - chunk.offset) as usize;
        #[cfg(feature = "buf_debug")]
        let data_slice = &chunk.data[st..];
        #[cfg(not(feature = "buf_debug"))]
        let data_slice = unsafe { &chunk.data.get_unchecked(st..chunk.data.len()) };
        //
        if data_slice.len() >= SIZE {
            #[cfg(feature = "buf_debug")]
            let slice = &data_slice[0..SIZE];
            #[cfg(not(feature = "buf_debug"))]
            let slice = unsafe { &data_slice.get_unchecked(0..SIZE) };
            //
            let val = {
                let mut ary = [0u8; SIZE];
                ary.copy_from_slice(slice);
                u16::from_le_bytes(ary)
            };
            //
            self.pos += SIZE as u64;
            Ok(val)
        } else {
            let mut buf = [0u8; SIZE];
            let _ = self.read_exact(&mut buf[..SIZE])?;
            Ok(u16::from_le_bytes(buf))
        }
    }
    #[inline]
    fn read_u32_le(&mut self) -> Result<u32> {
        const SIZE: usize = 4;
        let curr = self.pos;
        let chunk = self.fetch_chunk(curr)?;
        let st = (curr - chunk.offset) as usize;
        #[cfg(feature = "buf_debug")]
        let data_slice = &chunk.data[st..];
        #[cfg(not(feature = "buf_debug"))]
        let data_slice = unsafe { &chunk.data.get_unchecked(st..chunk.data.len()) };
        //
        if data_slice.len() >= SIZE {
            #[cfg(feature = "buf_debug")]
            let slice = &data_slice[0..SIZE];
            #[cfg(not(feature = "buf_debug"))]
            let slice = unsafe { &data_slice.get_unchecked(0..SIZE) };
            //
            let val = {
                let mut ary = [0u8; SIZE];
                ary.copy_from_slice(slice);
                u32::from_le_bytes(ary)
            };
            //
            self.pos += SIZE as u64;
            Ok(val)
        } else {
            let mut buf = [0u8; SIZE];
            let _ = self.read_exact(&mut buf[..SIZE])?;
            Ok(u32::from_le_bytes(buf))
        }
    }
    #[inline]
    fn read_u64_le(&mut self) -> Result<u64> {
        const SIZE: usize = 8;
        let curr = self.pos;
        let chunk = self.fetch_chunk(curr)?;
        let st = (curr - chunk.offset) as usize;
        #[cfg(feature = "buf_debug")]
        let data_slice = &chunk.data[st..];
        #[cfg(not(feature = "buf_debug"))]
        let data_slice = unsafe { &chunk.data.get_unchecked(st..chunk.data.len()) };
        //
        if data_slice.len() >= SIZE {
            #[cfg(feature = "buf_debug")]
            let slice = &data_slice[0..SIZE];
            #[cfg(not(feature = "buf_debug"))]
            let slice = unsafe { &data_slice.get_unchecked(0..SIZE) };
            //
            let val = {
                let mut ary = [0u8; SIZE];
                ary.copy_from_slice(slice);
                u64::from_le_bytes(ary)
            };
            //
            self.pos += SIZE as u64;
            Ok(val)
        } else {
            let mut buf = [0u8; SIZE];
            let _ = self.read_exact(&mut buf[..SIZE])?;
            Ok(u64::from_le_bytes(buf))
        }
    }
    /// Read maximum 8 bytes with a fast routine and return little endian u64.
    #[inline]
    fn read_max_8_bytes(&mut self, size: usize) -> Result<u64> {
        debug_assert!(size <= 8, "size: {} <= 8", size,);
        let curr = self.pos;
        let max_8_bytes = {
            let chunk = self.fetch_chunk(curr)?;
            let st = (curr - chunk.offset) as usize;
            #[cfg(feature = "buf_debug")]
            let data_slice = &chunk.data[st..];
            #[cfg(not(feature = "buf_debug"))]
            let data_slice = unsafe { &chunk.data.get_unchecked(st..chunk.data.len()) };
            //
            if data_slice.len() >= 8 {
                let val = {
                    let mut val = 0u64;
                    let mut i = size as i32 - 1;
                    while i >= 0 {
                        let byte = unsafe { *data_slice.get_unchecked(i as usize) };
                        val = val << 8 | byte as u64;
                        i -= 1;
                    }
                    val
                };
                //
                /*
                let val = {
                    let mut ary = [0u8; 8];
                    //
                    #[cfg(feature = "buf_debug")]
                    let dest = &mut ary[..size];
                    #[cfg(not(feature = "buf_debug"))]
                    let dest = unsafe { ary.get_unchecked_mut(0..size) };
                    #[cfg(feature = "buf_debug")]
                    let src = &data_slice[..size];
                    #[cfg(not(feature = "buf_debug"))]
                    let src = unsafe { data_slice.get_unchecked(0..size) };
                    //
                    dest.copy_from_slice(src);
                    u64::from_le_bytes(ary)
                };
                */
                //
                self.pos += size as u64;
                val
            } else {
                let mut buf = [0u8; 8];
                let _ = self.read_exact(&mut buf[..size])?;
                u64::from_le_bytes(buf)
            }
        };
        Ok(max_8_bytes)
    }
    /// Read small size bytes with a fast routine. The small size is less than chunk size.
    #[inline]
    fn read_exact_small(&mut self, buf: &mut [u8]) -> Result<()> {
        debug_assert!(
            buf.len() <= self.chunk_size,
            "buf.len(): {} <= {}",
            buf.len(),
            self.chunk_size
        );
        let curr = self.pos;
        let chunk = self.fetch_chunk(curr)?;
        let buf_len = buf.len();
        let st = (curr - chunk.offset) as usize;
        if st + buf_len <= chunk.data.len() {
            #[cfg(feature = "buf_debug")]
            buf.copy_from_slice(&chunk.data[st..(st + buf_len)]);
            #[cfg(not(feature = "buf_debug"))]
            buf.copy_from_slice(&chunk.data[st..(st + buf_len)]);
            self.pos += buf_len as u64;
            Ok(())
        } else {
            self.read_exact(buf)?;
            Ok(())
        }
    }
    /// Read small size bytes and return MaybeSlice.
    #[inline]
    fn read_exact_maybeslice(&mut self, size: usize) -> Result<MaybeSlice<'_>> {
        let (idx, st, data_sz) = {
            let curr = self.pos;
            let _ = self.fetch_chunk(curr)?;
            if let Some((offset, idx)) = self.fetch_cache {
                let st = (curr - offset) as usize;
                #[cfg(feature = "buf_debug")]
                let data_len = self.chunks[idx].data.len();
                #[cfg(not(feature = "buf_debug"))]
                let data_len = unsafe { self.chunks.get_unchecked(idx).data.len() };
                (idx, st, data_len - st)
            } else {
                (0, 0, 0)
            }
        };
        if size <= data_sz {
            self.pos += size as u64;
            //
            #[cfg(feature = "buf_debug")]
            let slice = &self.chunks[idx].data[st..(st + size)];
            #[cfg(not(feature = "buf_debug"))]
            let slice = unsafe {
                &(self.chunks.get_unchecked(idx))
                    .data
                    .get_unchecked(st..(st + size))
            };
            //
            return Ok(MaybeSlice::Slice(slice));
        }
        self.read_exact_maybeslice_vec_(size)
    }
}

/// Write small bytes less than chunk size.
pub trait SmallWrite {
    /// Write one byte with a fast routine.
    fn write_u8(&mut self, val: u8) -> Result<()>;
    /// Write a little endian u16 with a fast routine.
    fn write_u16_le(&mut self, val: u16) -> Result<()>;
    /// Write a little endian u32 with a fast routine.
    fn write_u32_le(&mut self, val: u32) -> Result<()>;
    /// Write a little endian u64 with a fast routine.
    fn write_u64_le(&mut self, val: u64) -> Result<()>;
    /// Write many little endian u64 with a fast routine.
    fn write_u64_le_slice(&mut self, val_slice: &[u64]) -> Result<()>;
    /// Write double many little endian u64 with a fast routine.
    fn write_u64_le_slice2(&mut self, val_slice1: &[u64], val_slice2: &[u64]) -> Result<()>;
    /// Write small size bytes with a fast routine. The small size is less than chunk size.
    fn write_all_small(&mut self, buf: &[u8]) -> Result<()>;
    /// Write `0u8` of length `size` with a fast routine.
    fn write_zero(&mut self, size: u32) -> Result<()>;
}

impl SmallWrite for BufFile {
    #[inline]
    fn write_u8(&mut self, val: u8) -> Result<()> {
        const SIZE: usize = 1;
        {
            let curr = self.pos;
            let chunk = self.fetch_chunk(curr)?;
            let st = (curr - chunk.offset) as usize;
            if st + SIZE <= chunk.data.len() {
                chunk.dirty = true;
                //
                #[cfg(feature = "buf_debug")]
                let dest = &mut chunk.data[st..(st + SIZE)];
                #[cfg(not(feature = "buf_debug"))]
                let dest = unsafe { chunk.data.get_unchecked_mut(st..(st + SIZE)) };
                //
                dest.copy_from_slice(&val.to_le_bytes());
                self.pos += SIZE as u64;
                if self.end < self.pos {
                    self.end = self.pos;
                }
                return Ok(());
            }
        }
        {
            let mut buf = [0u8; SIZE];
            buf.copy_from_slice(&val.to_le_bytes());
            self.write_all(buf.as_slice())
        }
    }
    #[inline]
    fn write_u16_le(&mut self, val: u16) -> Result<()> {
        const SIZE: usize = 2;
        {
            let curr = self.pos;
            let chunk = self.fetch_chunk(curr)?;
            let st = (curr - chunk.offset) as usize;
            if st + SIZE <= chunk.data.len() {
                chunk.dirty = true;
                //
                #[cfg(feature = "buf_debug")]
                let dest = &mut chunk.data[st..(st + SIZE)];
                #[cfg(not(feature = "buf_debug"))]
                let dest = unsafe { chunk.data.get_unchecked_mut(st..(st + SIZE)) };
                //
                dest.copy_from_slice(&val.to_le_bytes());
                self.pos += SIZE as u64;
                if self.end < self.pos {
                    self.end = self.pos;
                }
                return Ok(());
            }
        }
        {
            let mut buf = [0u8; SIZE];
            buf.copy_from_slice(&val.to_le_bytes());
            self.write_all(buf.as_slice())
        }
    }
    #[inline]
    fn write_u32_le(&mut self, val: u32) -> Result<()> {
        const SIZE: usize = 4;
        {
            let curr = self.pos;
            let chunk = self.fetch_chunk(curr)?;
            let st = (curr - chunk.offset) as usize;
            if st + SIZE <= chunk.data.len() {
                chunk.dirty = true;
                //
                #[cfg(feature = "buf_debug")]
                let dest = &mut chunk.data[st..(st + SIZE)];
                #[cfg(not(feature = "buf_debug"))]
                let dest = unsafe { chunk.data.get_unchecked_mut(st..(st + SIZE)) };
                //
                dest.copy_from_slice(&val.to_le_bytes());
                self.pos += SIZE as u64;
                if self.end < self.pos {
                    self.end = self.pos;
                }
                return Ok(());
            }
        }
        {
            let mut buf = [0u8; SIZE];
            buf.copy_from_slice(&val.to_le_bytes());
            self.write_all(buf.as_slice())
        }
    }
    #[inline]
    fn write_u64_le(&mut self, val: u64) -> Result<()> {
        const SIZE: usize = 8;
        {
            let curr = self.pos;
            let chunk = self.fetch_chunk(curr)?;
            let st = (curr - chunk.offset) as usize;
            if st + SIZE <= chunk.data.len() {
                chunk.dirty = true;
                //
                #[cfg(feature = "buf_debug")]
                let dest = &mut chunk.data[st..(st + SIZE)];
                #[cfg(not(feature = "buf_debug"))]
                let dest = unsafe { chunk.data.get_unchecked_mut(st..(st + SIZE)) };
                //
                dest.copy_from_slice(&val.to_le_bytes());
                self.pos += SIZE as u64;
                if self.end < self.pos {
                    self.end = self.pos;
                }
                return Ok(());
            }
        }
        {
            let mut buf = [0u8; SIZE];
            buf.copy_from_slice(&val.to_le_bytes());
            return self.write_all(buf.as_slice());
        }
    }
    #[inline]
    fn write_u64_le_slice(&mut self, val_slice: &[u64]) -> Result<()> {
        let size = 8 * val_slice.len();
        {
            let curr = self.pos;
            let chunk = self.fetch_chunk(curr)?;
            let st = (curr - chunk.offset) as usize;
            if st + size <= chunk.data.len() {
                chunk.dirty = true;
                for i in 0..val_slice.len() {
                    #[cfg(feature = "buf_debug")]
                    {
                        let dest = &mut chunk.data[(st + i * 8)..(st + (i + 1) * 8)];
                        let val = &val_slice[i];
                        dest.copy_from_slice(&val.to_le_bytes());
                    }
                    #[cfg(not(feature = "buf_debug"))]
                    {
                        let dest = unsafe {
                            chunk
                                .data
                                .get_unchecked_mut((st + i * 8)..(st + (i + 1) * 8))
                        };
                        let val = unsafe { val_slice.get_unchecked(i) };
                        dest.copy_from_slice(&val.to_le_bytes());
                    }
                }
                self.pos += size as u64;
                if self.end < self.pos {
                    self.end = self.pos;
                }
                return Ok(());
            }
        }
        {
            let mut buf = vec![0u8; size];
            for i in 0..val_slice.len() {
                #[cfg(feature = "buf_debug")]
                {
                    let dest = &mut buf[i * 8..(i + 1) * 8];
                    let val = &val_slice[i];
                    dest.copy_from_slice(&val.to_le_bytes());
                }
                #[cfg(not(feature = "buf_debug"))]
                {
                    let dest = unsafe { buf.get_unchecked_mut(i * 8..(i + 1) * 8) };
                    let val = unsafe { val_slice.get_unchecked(i) };
                    dest.copy_from_slice(&val.to_le_bytes());
                }
            }
            self.write_all(buf.as_slice())
        }
    }
    #[inline]
    fn write_u64_le_slice2(&mut self, val_slice1: &[u64], val_slice2: &[u64]) -> Result<()> {
        let size = 8 * (val_slice1.len() + val_slice2.len());
        {
            let curr = self.pos;
            let chunk = self.fetch_chunk(curr)?;
            let st = (curr - chunk.offset) as usize;
            if st + size <= chunk.data.len() {
                chunk.dirty = true;
                for i in 0..val_slice1.len() {
                    #[cfg(feature = "buf_debug")]
                    {
                        let dest = &mut chunk.data[(st + i * 8)..(st + (i + 1) * 8)];
                        let val = &val_slice1[i];
                        dest.copy_from_slice(&val.to_le_bytes());
                    }
                    #[cfg(not(feature = "buf_debug"))]
                    {
                        let dest = unsafe {
                            chunk
                                .data
                                .get_unchecked_mut((st + i * 8)..(st + (i + 1) * 8))
                        };
                        let val = unsafe { val_slice1.get_unchecked(i) };
                        dest.copy_from_slice(&val.to_le_bytes());
                    }
                }
                let st2 = st + 8 * val_slice1.len();
                for i in 0..val_slice2.len() {
                    #[cfg(feature = "buf_debug")]
                    {
                        let dest = &mut chunk.data[(st2 + i * 8)..(st2 + (i + 1) * 8)];
                        let val = &val_slice2[i];
                        dest.copy_from_slice(&val.to_le_bytes());
                    }
                    #[cfg(not(feature = "buf_debug"))]
                    {
                        let dest = unsafe {
                            chunk
                                .data
                                .get_unchecked_mut((st2 + i * 8)..(st2 + (i + 1) * 8))
                        };
                        let val = unsafe { val_slice2.get_unchecked(i) };
                        dest.copy_from_slice(&val.to_le_bytes());
                    }
                }
                self.pos += size as u64;
                if self.end < self.pos {
                    self.end = self.pos;
                }
                return Ok(());
            }
        }
        {
            let mut buf = vec![0u8; size];
            for i in 0..val_slice1.len() {
                #[cfg(feature = "buf_debug")]
                {
                    let dest = &mut buf[i * 8..(i + 1) * 8];
                    let val = &val_slice1[i];
                    dest.copy_from_slice(&val.to_le_bytes());
                }
                #[cfg(not(feature = "buf_debug"))]
                {
                    let dest = unsafe { buf.get_unchecked_mut(i * 8..(i + 1) * 8) };
                    let val = unsafe { val_slice1.get_unchecked(i) };
                    dest.copy_from_slice(&val.to_le_bytes());
                }
            }
            let st2 = 8 * val_slice1.len();
            for i in 0..val_slice2.len() {
                #[cfg(feature = "buf_debug")]
                {
                    let dest = &mut buf[(st2 + i * 8)..(st2 + (i + 1) * 8)];
                    let val = &val_slice2[i];
                    dest.copy_from_slice(&val.to_le_bytes());
                }
                #[cfg(not(feature = "buf_debug"))]
                {
                    let dest = unsafe { buf.get_unchecked_mut((st2 + i * 8)..(st2 + (i + 1) * 8)) };
                    let val = unsafe { val_slice2.get_unchecked(i) };
                    dest.copy_from_slice(&val.to_le_bytes());
                }
            }
            self.write_all(buf.as_slice())
        }
    }
    #[inline]
    fn write_all_small(&mut self, buf: &[u8]) -> Result<()> {
        debug_assert!(
            buf.len() <= self.chunk_size,
            "buf.len(): {} <= {}",
            buf.len(),
            self.chunk_size
        );
        {
            let curr = self.pos;
            let chunk = self.fetch_chunk(curr)?;
            let buf_len = buf.len();
            let st = (curr - chunk.offset) as usize;
            if st + buf_len <= chunk.data.len() {
                chunk.dirty = true;
                //
                #[cfg(feature = "buf_debug")]
                let dest = &mut chunk.data[st..(st + buf_len)];
                #[cfg(not(feature = "buf_debug"))]
                let dest = unsafe { chunk.data.get_unchecked_mut(st..(st + buf_len)) };
                //
                dest.copy_from_slice(buf);
                self.pos += buf_len as u64;
                if self.end < self.pos {
                    self.end = self.pos;
                }
                return Ok(());
            }
        }
        self.write_all(buf)
    }
    #[inline]
    fn write_zero(&mut self, size: u32) -> Result<()> {
        let size = size as usize;
        {
            let curr = self.pos;
            let chunk = self.fetch_chunk(curr)?;
            let st = (curr - chunk.offset) as usize;
            if st + size <= chunk.data.len() {
                chunk.dirty = true;
                //
                #[cfg(feature = "buf_debug")]
                let dest = &mut chunk.data[st..(st + size)];
                #[cfg(not(feature = "buf_debug"))]
                let dest = unsafe { chunk.data.get_unchecked_mut(st..(st + size)) };
                //
                dest.fill(0u8);
                self.pos += size as u64;
                if self.end < self.pos {
                    self.end = self.pos;
                }
                return Ok(());
            }
        }
        self.write_zero_0_(size)
    }
}

/// Auto buffer size with per mille of the file size.
#[cfg(feature = "buf_auto_buf_size")]
#[derive(Debug)]
struct AutoBufferSize(u16);

#[cfg(feature = "buf_auto_buf_size")]
impl AutoBufferSize {
    pub fn with_per_mille(per_mille: u16) -> Self {
        Self(per_mille)
    }
    #[inline]
    fn buffer_size(&self, file_size: u64) -> Result<usize> {
        let per_mille = self.0;
        if per_mille > 0 {
            let val = if per_mille >= 1000 {
                file_size
            } else {
                (file_size / 1000) * per_mille as u64
            };
            if val > 8 * 4 * 1024 {
                Ok(val as usize)
            } else {
                Ok(8 * 4 * 1024)
            }
        } else {
            Ok(8 * 4 * 1024)
        }
    }
}

/// Chunk size MUST be a power of 2.
const CHUNK_SIZE: u32 = 1024 * 4;

#[cfg(not(feature = "buf_auto_buf_size"))]
const DEFAULT_NUM_CHUNKS: u16 = 16;

#[cfg(feature = "buf_auto_buf_size")]
const DEFAULT_PER_MILLE: u16 = 20;

/// Chunk buffer for reading or writing.
#[derive(Debug)]
struct Chunk {
    /// chunk data. it is a buffer for reading or writing.
    pub data: Vec<u8>,
    /// chunk offset. it is a offset from start of the file.
    offset: u64,
    /// dirty flag. we should write the chunk to the file.
    dirty: bool,
    /// uses counter. counts up if we read or write chunk.
    #[cfg(not(feature = "buf_overf_rem_all"))]
    uses: u32,
}

impl Chunk {
    fn new<U: Seek + Read>(
        offset: u64,
        end_pos: u64,
        chunk_size: usize,
        file: &mut U,
    ) -> Result<Chunk> {
        file.seek(SeekFrom::Start(offset))?;
        let mut data = vec![0u8; chunk_size];
        if offset != end_pos {
            let end_off = (end_pos - offset) as usize;
            #[cfg(feature = "buf_debug")]
            let buf = if end_off >= chunk_size {
                &mut data[0..]
            } else {
                &mut data[0..end_off]
            };
            #[cfg(not(feature = "buf_debug"))]
            let buf = if end_off >= chunk_size {
                unsafe { data.get_unchecked_mut(0..chunk_size) }
            } else {
                unsafe { data.get_unchecked_mut(0..end_off) }
            };
            if let Err(err) = file.read_exact(buf) {
                let _ = std::marker::PhantomData::<i32>;
                return Err(err);
            }
        }
        Ok(Chunk {
            data,
            offset,
            dirty: false,
            #[cfg(not(feature = "buf_overf_rem_all"))]
            uses: 0,
        })
    }
    //
    #[cfg(not(feature = "buf_overf_rem"))]
    fn read_inplace<U: Seek + Read + Write>(
        &mut self,
        offset: u64,
        end_pos: u64,
        file: &mut U,
    ) -> Result<()> {
        let chunk_size = self.data.len();
        //
        file.seek(SeekFrom::Start(offset))?;
        let data = &mut self.data;
        data.fill(0u8);
        if offset != end_pos {
            let end_off = (end_pos - offset) as usize;
            let _data_len = self.data.len();
            #[cfg(feature = "buf_debug")]
            let buf = if end_off >= chunk_size {
                &mut data[0..]
            } else {
                &mut data[0..end_off]
            };
            #[cfg(not(feature = "buf_debug"))]
            let buf = if end_off >= chunk_size {
                unsafe { self.data.get_unchecked(0.._data_len) }
            } else {
                unsafe { self.data.get_unchecked(0..end_off) }
            };
            if let Err(err) = file.read_exact(buf) {
                let _ = std::marker::PhantomData::<i32>;
                return Err(err);
            }
        }
        //
        self.dirty = false;
        self.offset = offset;
        self.uses = 0;
        //
        Ok(())
    }
    //
    fn write<U: Seek + Read + Write>(&mut self, end_pos: u64, file: &mut U) -> Result<()> {
        if !self.dirty {
            return Ok(());
        }
        if self.offset > end_pos {
            return Ok(());
        }
        file.seek(SeekFrom::Start(self.offset))?;
        let end_off = (end_pos - self.offset) as usize;
        let data_len = self.data.len();
        #[cfg(feature = "buf_debug")]
        let buf = if end_off >= data_len {
            &self.data[0..]
        } else {
            &self.data[0..end_off]
        };
        #[cfg(not(feature = "buf_debug"))]
        let buf = if end_off >= data_len {
            unsafe { self.data.get_unchecked(0..data_len) }
        } else {
            unsafe { self.data.get_unchecked(0..end_off) }
        };
        match file.write_all(buf) {
            Ok(()) => {
                self.dirty = false;
                Ok(())
            }
            Err(err) => {
                let _ = std::marker::PhantomData::<i32>;
                Err(err)
            }
        }
    }
}

/// MyHasher
/// https://en.wikipedia.org/wiki/Xorshift
#[derive(Debug, Default)]
struct MyHasher(u64);

impl Hasher for MyHasher {
    fn write(&mut self, bytes: &[u8]) {
        let bytes_len = bytes.len();
        if bytes_len == 8 {
            let mut ary = [0u8; 8];
            ary.copy_from_slice(bytes);
            let mut a = u64::from_ne_bytes(ary);
            a = a ^ a >> 12;
            a = a ^ a << 25;
            a = a ^ a >> 27;
            self.0 = a;
        } else {
            for i in 0..bytes.len() {
                let a = unsafe { *bytes.get_unchecked(i) };
                self.0 = self.0.wrapping_add(a as u64);
            }
        }
    }
    fn write_u64(&mut self, val: u64) {
        let mut a = val;
        a = a ^ a >> 12;
        a = a ^ a << 25;
        a = a ^ a >> 27;
        self.0 = a;
    }
    fn finish(&self) -> u64 {
        self.0
    }
}

/// Implements key-value sorted vec.
/// the key is the offset from start the file.
/// the value is the index of BufFile::data.
#[derive(Debug)]
struct OffsetIndex {
    #[cfg(not(feature = "buf_hash_turbo"))]
    vec: Vec<(u64, usize)>,
    #[cfg(feature = "buf_hash_turbo")]
    #[cfg(not(feature = "buf_myhash"))]
    map: HashMap<u64, usize>,
    #[cfg(feature = "buf_hash_turbo")]
    #[cfg(feature = "buf_myhash")]
    map: HashMap<u64, usize, BuildHasherDefault<MyHasher>>,
}
impl OffsetIndex {
    fn with_capacity(_cap: usize) -> Self {
        Self {
            #[cfg(not(feature = "buf_hash_turbo"))]
            vec: Vec::with_capacity(_cap),
            #[cfg(feature = "buf_hash_turbo")]
            #[cfg(not(feature = "buf_myhash"))]
            map: HashMap::with_capacity(_cap),
            #[cfg(feature = "buf_hash_turbo")]
            #[cfg(feature = "buf_myhash")]
            map: HashMap::with_capacity_and_hasher(_cap, Default::default()),
        }
    }
    #[inline]
    fn get(&mut self, offset: &u64) -> Option<usize> {
        #[cfg(feature = "buf_hash_turbo")]
        {
            self.map.get(offset).copied()
        }
        #[cfg(not(feature = "buf_hash_turbo"))]
        {
            let slice = &self.vec;
            if let Ok(x) = slice.binary_search_by(|a| a.0.cmp(offset)) {
                #[cfg(feature = "buf_debug")]
                let val = self.vec[x].1;
                #[cfg(not(feature = "buf_debug"))]
                let val = unsafe { slice.get_unchecked(x).1 };
                //
                Some(val)
            } else {
                None
            }
        }
    }
    #[inline]
    fn insert(&mut self, offset: &u64, idx: usize) {
        #[cfg(feature = "buf_hash_turbo")]
        {
            let _ = self.map.insert(*offset, idx);
        }
        #[cfg(not(feature = "buf_hash_turbo"))]
        {
            match self.vec.binary_search_by(|a| a.0.cmp(offset)) {
                Ok(x) => {
                    self.vec[x].1 = idx;
                }
                Err(x) => {
                    self.vec.insert(x, (*offset, idx));
                }
            }
        }
    }
    fn remove(&mut self, offset: &u64) -> Option<usize> {
        #[cfg(feature = "buf_hash_turbo")]
        {
            let r = self.map.remove(offset);
            r
        }
        #[cfg(not(feature = "buf_hash_turbo"))]
        {
            match self.vec.binary_search_by(|a| a.0.cmp(offset)) {
                Ok(x) => Some(self.vec.remove(x).1),
                Err(_x) => None,
            }
        }
    }
    #[inline]
    fn clear(&mut self) {
        #[cfg(feature = "buf_hash_turbo")]
        {
            self.map.clear();
        }
        #[cfg(not(feature = "buf_hash_turbo"))]
        {
            self.vec.clear();
        }
    }
}

/// Generic random access buffer.
#[derive(Debug)]
pub struct RaBuf<T: Seek + Read + Write> {
    /// The name of rabuf for debugging.
    name: String,
    /// The maximum number of chunk
    max_num_chunks: usize,
    /// Chunk buffer size in bytes.
    chunk_size: usize,
    /// Chunk offset mask.
    chunk_mask: u64,
    /// Contains the actual chunks
    chunks: Vec<Chunk>,
    /// Used to quickly map a file index to an array index (to index self.dat)
    map: OffsetIndex,
    /// The file to be written to and read from
    file: T,
    /// The current position of the file.
    pos: u64,
    /// The file offset that is the end of the file.
    end: u64,
    //
    fetch_cache: Option<(u64, usize)>,
    //
    #[cfg(feature = "buf_lru")]
    uses_cnt: u32,
    //
    /// a minimum uses counter, but grater than 0.
    #[cfg(feature = "buf_stats")]
    stats_min_uses: u32,
    /// a maximum uses counter
    #[cfg(feature = "buf_stats")]
    stats_max_uses: u32,
    /// a per mille for the file size.
    #[cfg(feature = "buf_auto_buf_size")]
    auto_buf_size: Option<AutoBufferSize>,
    /// a count of fc hits.
    #[cfg(feature = "buf_print_hits")]
    count_of_hits_fc: u64,
    /// a count of hits.
    #[cfg(feature = "buf_print_hits")]
    count_of_hits: u64,
    /// a count of miss.
    #[cfg(feature = "buf_print_hits")]
    count_of_miss: u64,
}

// ref.) http://graphics.stanford.edu/~seander/bithacks.html#RoundUpPowerOf2

/// Round up power of 2.
#[inline]
pub fn roundup_powerof2(mut v: u32) -> u32 {
    v -= 1;
    v |= v >> 1;
    v |= v >> 2;
    v |= v >> 4;
    v |= v >> 8;
    v |= v >> 16;
    v += 1;
    v
}

// public implements
impl<T: Seek + Read + Write> RaBuf<T> {
    /// Creates a new BufFile.
    /// number of chunk: 16, chunk size: 4096
    pub fn new(name: &str, file: T) -> Result<RaBuf<T>> {
        #[cfg(not(feature = "buf_auto_buf_size"))]
        {
            Self::with_capacity(name, file, CHUNK_SIZE, DEFAULT_NUM_CHUNKS)
        }
        #[cfg(feature = "buf_auto_buf_size")]
        {
            Self::with_per_mille(name, file, CHUNK_SIZE, DEFAULT_PER_MILLE)
        }
    }
    /// Creates a new BufFile with the specified number of chunks.
    /// chunk_size is MUST power of 2.
    pub fn with_capacity(
        name: &str,
        mut file: T,
        chunk_size: u32,
        max_num_chunks: u16,
    ) -> Result<RaBuf<T>> {
        debug_assert!(chunk_size == roundup_powerof2(chunk_size));
        debug_assert!(max_num_chunks > 0);
        let max_num_chunks = max_num_chunks as usize;
        let chunk_mask = !(chunk_size as u64 - 1);
        let chunk_size = chunk_size as usize;
        let end = file.seek(SeekFrom::End(0))?;
        file.seek(SeekFrom::Start(0))?;
        //
        Ok(Self {
            name: name.to_string(),
            max_num_chunks,
            chunk_size,
            chunk_mask,
            chunks: Vec::with_capacity(max_num_chunks),
            map: OffsetIndex::with_capacity(max_num_chunks),
            file,
            pos: 0,
            end,
            fetch_cache: None,
            #[cfg(feature = "buf_lru")]
            uses_cnt: 0,
            #[cfg(feature = "buf_stats")]
            stats_min_uses: 0,
            #[cfg(feature = "buf_stats")]
            stats_max_uses: 0,
            #[cfg(feature = "buf_auto_buf_size")]
            auto_buf_size: None,
            #[cfg(feature = "buf_print_hits")]
            count_of_hits_fc: 0,
            #[cfg(feature = "buf_print_hits")]
            count_of_hits: 0,
            #[cfg(feature = "buf_print_hits")]
            count_of_miss: 0,
        })
    }
    /// Create a new BufFile with auto buffer size per mille of file size.
    /// chunk_size is MUST power of 2.
    #[cfg(feature = "buf_auto_buf_size")]
    pub fn with_per_mille(
        name: &str,
        mut file: T,
        chunk_size: u32,
        per_mille: u16,
    ) -> Result<RaBuf<T>> {
        debug_assert!(chunk_size == roundup_powerof2(chunk_size));
        let chunk_mask = !(chunk_size as u64 - 1);
        let chunk_size = chunk_size as usize;
        let auto_buf_size = AutoBufferSize::with_per_mille(per_mille);
        let end = file.seek(SeekFrom::End(0))?;
        let max_num_chunks = (auto_buf_size.buffer_size(end)? / chunk_size) + 1;
        file.seek(SeekFrom::Start(0))?;
        //
        Ok(Self {
            name: name.to_string(),
            max_num_chunks,
            chunk_size,
            chunk_mask,
            chunks: Vec::with_capacity(max_num_chunks),
            map: OffsetIndex::with_capacity(max_num_chunks),
            file,
            pos: 0,
            end,
            fetch_cache: None,
            #[cfg(feature = "buf_lru")]
            uses_cnt: 0,
            #[cfg(feature = "buf_stats")]
            stats_min_uses: 0,
            #[cfg(feature = "buf_stats")]
            stats_max_uses: 0,
            auto_buf_size: Some(auto_buf_size),
            #[cfg(feature = "buf_print_hits")]
            count_of_hits_fc: 0,
            #[cfg(feature = "buf_print_hits")]
            count_of_hits: 0,
            #[cfg(feature = "buf_print_hits")]
            count_of_miss: 0,
        })
    }
    /// Flush and clear all buffer chunks.
    #[inline]
    pub fn clear(&mut self) -> Result<()> {
        self.flush()?;
        self.fetch_cache = None;
        #[cfg(not(feature = "buf_pin_zero"))]
        {
            self.chunks.clear();
            self.map.clear();
        }
        #[cfg(feature = "buf_pin_zero")]
        {
            if let Some(idx) = self.map.get(&0) {
                let chunk_zero = self.chunks.remove(idx);
                self.chunks.clear();
                self.map.clear();
                self.chunks.push(chunk_zero);
                self.map.insert(&0, 0);
            } else {
                self.chunks.clear();
                self.map.clear();
            }
        }
        #[cfg(feature = "buf_lru")]
        {
            // clear LRU(: Least Reacently Used) counter
            self.uses_cnt = 0;
        }
        Ok(())
    }
    /// Name for debugging
    #[inline]
    pub fn name(&self) -> String {
        self.name.clone()
    }
    /// make preparation
    #[inline]
    pub fn prepare(&mut self, offset: u64) -> Result<()> {
        let _ = self.fetch_chunk(offset)?;
        Ok(())
    }
    ///
    #[cfg(feature = "buf_stats")]
    pub fn buf_stats(&self) -> Vec<(String, i64)> {
        let mut vec = Vec::new();
        vec.push((
            "BufFile.stats_min_uses".to_string(),
            self.stats_min_uses as i64,
        ));
        vec.push((
            "BufFile.stats_max_uses".to_string(),
            self.stats_max_uses as i64,
        ));
        vec
    }
}

impl<T: Seek + Read + Write> RaBuf<T> {
    #[cfg(feature = "buf_auto_buf_size")]
    #[inline]
    fn setup_auto_buf_size(&mut self) -> Result<()> {
        if let Some(ab_sz) = &self.auto_buf_size {
            let val = (ab_sz.buffer_size(self.end)? / self.chunk_size) + 1;
            if val > self.chunks.len() {
                self.max_num_chunks = val;
            }
        }
        Ok(())
    }
    #[inline]
    fn touch(&mut self, _chunk_idx: usize) {
        #[cfg(feature = "buf_overf_rem")]
        {
            // nothing todo
        }
        #[cfg(not(feature = "buf_overf_rem"))]
        {
            #[cfg(not(feature = "buf_lru"))]
            {
                self.chunks[_chunk_idx].uses += 1;
            }
            #[cfg(feature = "buf_lru")]
            {
                self.uses_cnt += 1;
                self.chunks[_chunk_idx].uses = self.uses_cnt;
            }
        }
    }
    //
    #[inline]
    fn fetch_chunk(&mut self, offset: u64) -> Result<&mut Chunk> {
        let offset = offset & self.chunk_mask;
        if let Some((off, idx)) = self.fetch_cache {
            if off == offset {
                #[cfg(feature = "buf_print_hits")]
                {
                    self.count_of_hits_fc += 1;
                }
                self.touch(idx);
                //
                #[cfg(feature = "buf_debug")]
                let chunk_mut = &mut self.chunks[idx];
                #[cfg(not(feature = "buf_debug"))]
                let chunk_mut = unsafe { self.chunks.get_unchecked_mut(idx) };
                //
                return Ok(chunk_mut);
            }
        }
        self.fetch_chunk_0_(offset)
    }
    fn fetch_chunk_0_(&mut self, offset: u64) -> Result<&mut Chunk> {
        let idx = if let Some(x) = self.map.get(&offset) {
            #[cfg(feature = "buf_print_hits")]
            {
                self.count_of_hits += 1;
            }
            x
        } else {
            #[cfg(feature = "buf_print_hits")]
            {
                self.count_of_miss += 1;
            }
            self.add_chunk(offset)?
        };
        self.fetch_cache = Some((offset, idx));
        self.touch(idx);
        #[cfg(feature = "buf_debug")]
        let chunk_mut = &mut self.chunks[idx];
        #[cfg(not(feature = "buf_debug"))]
        let chunk_mut = unsafe { self.chunks.get_unchecked_mut(idx) };
        //
        Ok(chunk_mut)
    }
    //
    fn add_chunk(&mut self, offset: u64) -> Result<usize> {
        #[cfg(feature = "buf_auto_buf_size")]
        if self.chunks.len() == self.max_num_chunks {
            self.setup_auto_buf_size()?;
        }
        self.fetch_cache = None;
        if self.chunks.len() < self.max_num_chunks {
            let new_idx = self.chunks.len();
            match Chunk::new(offset, self.end, self.chunk_size, &mut self.file) {
                Ok(x) => {
                    self.map.insert(&offset, new_idx);
                    self.chunks.push(x);
                    Ok(new_idx)
                }
                Err(e) => Err(e),
            }
        } else {
            #[cfg(feature = "buf_overf_rem")]
            {
                self.remove_chunks()?;
                self.add_chunk(offset)
            }
            #[cfg(not(feature = "buf_overf_rem"))]
            {
                // LFU: Least Frequently Used
                let min_idx = {
                    // find the minimum uses counter.
                    let mut min_idx = 0;
                    let mut min_uses = self.chunks[min_idx].uses;
                    if min_uses != 0 {
                        for i in 1..self.max_num_chunks {
                            if self.chunks[i].uses < min_uses {
                                min_idx = i;
                                min_uses = self.chunks[min_idx].uses;
                                if min_uses == 0 {
                                    break;
                                }
                            } else {
                                #[cfg(feature = "buf_stats")]
                                {
                                    if self.chunks[i].uses > self.stats_max_uses {
                                        self.stats_max_uses = self.chunks[i].uses;
                                    }
                                }
                            }
                        }
                    }
                    #[cfg(feature = "buf_stats")]
                    {
                        if min_uses > 0 && min_uses < self.stats_min_uses {
                            self.stats_min_uses = min_uses;
                        }
                    }
                    // clear all uses counter
                    self.chunks.iter_mut().for_each(|chunk| {
                        chunk.uses = 0;
                    });
                    #[cfg(feature = "buf_lru")]
                    {
                        // clear LRU(: Least Reacently Used) counter
                        self.uses_cnt = 0;
                    }
                    min_idx
                };
                // Make a new chunk, write the old chunk to disk, replace old chunk
                self.chunks[min_idx].write(self.end, &mut self.file)?;
                self.map.remove(&self.chunks[min_idx].offset);
                self.map.insert(offset, min_idx);
                self.chunks[min_idx].read_inplace(offset, self.end, &mut self.file)?;
                #[cfg(feature = "buf_auto_buf_size")]
                self.setup_auto_buf_size()?;
                Ok(min_idx)
            }
        }
    }
    //
    #[cfg(all(feature = "buf_overf_rem", feature = "buf_overf_rem_all"))]
    fn remove_chunks(&mut self) -> Result<()> {
        self.clear()?;
        #[cfg(feature = "buf_auto_buf_size")]
        self.setup_auto_buf_size()?;
        Ok(())
    }
    #[cfg(all(feature = "buf_overf_rem", feature = "buf_overf_rem_half"))]
    fn remove_chunks(&mut self) -> Result<()> {
        // the LFU/LRU half clear
        let mut vec: Vec<(usize, u32)> = self
            .chunks
            .iter()
            .enumerate()
            .map(|(idx, chunk)| (idx, chunk.uses))
            .collect();
        vec.sort_by(|a, b| match b.1.cmp(&a.1) {
            std::cmp::Ordering::Equal => b.0.cmp(&a.0),
            std::cmp::Ordering::Less => std::cmp::Ordering::Less,
            std::cmp::Ordering::Greater => std::cmp::Ordering::Greater,
        });
        let half = vec.len() / 2;
        let _rest = vec.split_off(half);
        vec.sort_by(|a, b| a.0.cmp(&b.0));
        while let Some((idx, _uses)) = vec.pop() {
            let mut _chunk = self.chunks.remove(idx);
            _chunk.write(self.end, &mut self.file)?;
        }
        self.map.clear();
        // clear all uses counter
        let mut vec2: Vec<(u64, usize)> = Vec::new();
        self.chunks.iter_mut().enumerate().for_each(|(idx, chunk)| {
            vec2.push((chunk.offset, idx));
            chunk.uses = 0;
        });
        vec2.iter().for_each(|v| {
            self.map.insert(&v.0, v.1);
        });
        #[cfg(feature = "buf_auto_buf_size")]
        self.setup_auto_buf_size()?;
        #[cfg(feature = "buf_lru")]
        {
            // clear LRU(: Least Reacently Used) counter
            self.uses_cnt = 0;
        }
        Ok(())
    }
    //
    #[inline(never)]
    fn read_exact_maybeslice_vec_(&mut self, size: usize) -> Result<MaybeSlice<'_>> {
        let mut buf = vec![0u8; size];
        self.read_exact(&mut buf)?;
        Ok(MaybeSlice::Buffer(buf))
    }
    #[inline(never)]
    fn write_zero_0_(&mut self, size: usize) -> Result<()> {
        let buf = vec![0u8; size];
        self.write_all(&buf)
    }
}

impl<T: Seek + Read + Write> Read for RaBuf<T> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let curr = self.pos;
        let len = {
            let chunk = self.fetch_chunk(curr)?;
            let buf_len = buf.len();
            let st = (curr - chunk.offset) as usize;
            let _data_len = chunk.data.len();
            //
            #[cfg(feature = "buf_debug")]
            let data_slice = &chunk.data[st..];
            #[cfg(not(feature = "buf_debug"))]
            let data_slice = unsafe { chunk.data.get_unchecked(st.._data_len) };
            //
            let data_slice_len = data_slice.len();
            if buf_len <= data_slice_len {
                #[cfg(feature = "buf_debug")]
                let slice = &data_slice[..buf_len];
                #[cfg(not(feature = "buf_debug"))]
                let slice = unsafe { data_slice.get_unchecked(0..buf_len) };
                //
                buf.copy_from_slice(slice);
                buf_len
            } else {
                #[cfg(feature = "buf_debug")]
                let nallow_buf = &mut buf[..data_slice_len];
                #[cfg(not(feature = "buf_debug"))]
                let nallow_buf = unsafe { buf.get_unchecked_mut(0..data_slice_len) };
                //
                nallow_buf.copy_from_slice(data_slice);
                data_slice_len
            }
        };
        self.pos += len as u64;
        Ok(len)
    }
}

impl<T: Seek + Read + Write> Write for RaBuf<T> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let curr = self.pos;
        let len = {
            let chunk = self.fetch_chunk(curr)?;
            chunk.dirty = true;
            let buf_len = buf.len();
            let st = (curr - chunk.offset) as usize;
            let _data_len = chunk.data.len();
            #[cfg(feature = "buf_debug")]
            let data_slice = &mut chunk.data[st..];
            #[cfg(not(feature = "buf_debug"))]
            let data_slice = unsafe { chunk.data.get_unchecked_mut(st.._data_len) };
            //
            let data_slice_len = data_slice.len();
            if buf_len <= data_slice_len {
                #[cfg(feature = "buf_debug")]
                let slice = &mut data_slice[..buf_len];
                #[cfg(not(feature = "buf_debug"))]
                let slice = unsafe { data_slice.get_unchecked_mut(0..buf_len) };
                //
                slice.copy_from_slice(buf);
                buf_len
            } else {
                #[cfg(feature = "buf_debug")]
                let nallow_buf = &buf[..data_slice_len];
                #[cfg(not(feature = "buf_debug"))]
                let nallow_buf = unsafe { buf.get_unchecked(0..data_slice_len) };
                //
                data_slice.copy_from_slice(nallow_buf);
                data_slice_len
            }
        };
        self.pos += len as u64;
        if self.end < self.pos {
            self.end = self.pos;
        }
        Ok(len)
    }
    #[inline]
    fn flush(&mut self) -> Result<()> {
        #[cfg(feature = "buf_hash_turbo")]
        {
            let mut off_vec: Vec<u64> = self.map.map.keys().map(|&a| a).collect();
            off_vec.sort_unstable();
            for off in off_vec.iter() {
                let idx = self.map.map[off];
                //
                #[cfg(feature = "buf_debug")]
                let chunk = &mut self.chunks[idx];
                #[cfg(not(feature = "buf_debug"))]
                let chunk = unsafe { self.chunks.get_unchecked_mut(idx) };
                //
                chunk.write(self.end, &mut self.file)?;
            }
        }
        #[cfg(not(feature = "buf_hash_turbo"))]
        {
            for &(_, idx) in self.map.vec.iter() {
                #[cfg(feature = "buf_debug")]
                let chunk = &mut self.chunks[idx];
                #[cfg(not(feature = "buf_debug"))]
                let chunk = unsafe { self.chunks.get_unchecked_mut(idx) };
                //
                chunk.write(self.end, &mut self.file)?;
            }
        }
        Ok(())
    }
}

impl<T: Seek + Read + Write> Drop for RaBuf<T> {
    /// Write all of the chunks to disk before closing the file.
    fn drop(&mut self) {
        let _ = self.flush();
        #[cfg(feature = "buf_print_hits")]
        {
            let all = self.count_of_hits + self.count_of_miss;
            let all2 = self.count_of_hits_fc + all;
            let hits_fc = self.count_of_hits_fc as f64 * 100.0 / all2 as f64;
            let hits = self.count_of_hits as f64 * 100.0 / all as f64;
            let kb = self.chunk_size as f64 * self.max_num_chunks as f64 / (1024.0 * 1024.0);
            eprintln!(
                "rabuf \"{}\" cache hits_fc: {:4.1}%, hits: {:4.1}%, {:4.1}mib",
                self.name, hits_fc, hits, kb,
            );
        }
    }
}

//--
#[cfg(test)]
mod debug {
    use super::{BufFile, Chunk};
    //
    #[test]
    fn test_size_of() {
        #[cfg(target_pointer_width = "64")]
        {
            #[cfg(not(feature = "buf_hash_turbo"))]
            {
                #[cfg(not(feature = "buf_stats"))]
                {
                    assert_eq!(std::mem::size_of::<BufFile>(), 144);
                }
                #[cfg(feature = "buf_stats")]
                assert_eq!(std::mem::size_of::<BufFile>(), 128);
            }
            #[cfg(feature = "buf_hash_turbo")]
            {
                #[cfg(not(feature = "buf_myhash"))]
                {
                    #[cfg(not(feature = "buf_stats"))]
                    assert_eq!(std::mem::size_of::<BufFile>(), 192);
                    #[cfg(feature = "buf_stats")]
                    assert_eq!(std::mem::size_of::<BufFile>(), 200);
                }
                #[cfg(feature = "buf_myhash")]
                {
                    #[cfg(not(feature = "buf_stats"))]
                    assert_eq!(std::mem::size_of::<BufFile>(), 152);
                    #[cfg(feature = "buf_stats")]
                    assert_eq!(std::mem::size_of::<BufFile>(), 184);
                }
            }
            //
            assert_eq!(std::mem::size_of::<Chunk>(), 40);
            assert_eq!(std::mem::size_of::<(u64, usize)>(), 16);
            assert_eq!(std::mem::size_of::<Vec<Chunk>>(), 24);
            assert_eq!(std::mem::size_of::<Vec<u8>>(), 24);
        }
        #[cfg(target_pointer_width = "32")]
        {
            #[cfg(not(feature = "buf_hash_turbo"))]
            {
                #[cfg(not(any(feature = "buf_stats", feature = "buf_lru")))]
                {
                    #[cfg(not(target_arch = "arm"))]
                    {
                        #[cfg(not(any(
                            feature = "buf_overf_rem_all",
                            feature = "buf_overf_rem_half"
                        )))]
                        assert_eq!(std::mem::size_of::<BufFile>(), 80);
                        #[cfg(feature = "buf_overf_rem_half")]
                        assert_eq!(std::mem::size_of::<BufFile>(), 76);
                        #[cfg(feature = "buf_overf_rem_all")]
                        assert_eq!(std::mem::size_of::<BufFile>(), 92);
                    }
                    #[cfg(target_arch = "arm")]
                    assert_eq!(std::mem::size_of::<BufFile>(), 96);
                }
                #[cfg(all(feature = "buf_stats", feature = "buf_lru"))]
                {
                    #[cfg(not(target_arch = "arm"))]
                    assert_eq!(std::mem::size_of::<BufFile>(), 88);
                    #[cfg(target_arch = "arm")]
                    assert_eq!(std::mem::size_of::<BufFile>(), 96);
                }
                #[cfg(all(feature = "buf_stats", not(feature = "buf_lru")))]
                {
                    #[cfg(not(target_arch = "arm"))]
                    assert_eq!(std::mem::size_of::<BufFile>(), 84);
                    #[cfg(target_arch = "arm")]
                    assert_eq!(std::mem::size_of::<BufFile>(), 96);
                }
                #[cfg(all(not(feature = "buf_stats"), feature = "buf_lru"))]
                {
                    #[cfg(not(target_arch = "arm"))]
                    {
                        #[cfg(not(feature = "buf_overf_rem_half"))]
                        assert_eq!(std::mem::size_of::<BufFile>(), 80);
                        #[cfg(feature = "buf_overf_rem_half")]
                        assert_eq!(std::mem::size_of::<BufFile>(), 92);
                    }
                    #[cfg(target_arch = "arm")]
                    {
                        #[cfg(not(feature = "buf_overf_rem_half"))]
                        assert_eq!(std::mem::size_of::<BufFile>(), 80);
                        #[cfg(feature = "buf_overf_rem_half")]
                        assert_eq!(std::mem::size_of::<BufFile>(), 104);
                    }
                }
            }
            #[cfg(feature = "buf_hash_turbo")]
            {
                #[cfg(not(any(feature = "buf_stats", feature = "buf_lru")))]
                {
                    #[cfg(not(target_arch = "arm"))]
                    {
                        #[cfg(not(any(
                            feature = "buf_overf_rem_all",
                            feature = "buf_overf_rem_half"
                        )))]
                        assert_eq!(std::mem::size_of::<BufFile>(), 80);
                        #[cfg(feature = "buf_overf_rem_half")]
                        assert_eq!(std::mem::size_of::<BufFile>(), 76);
                        #[cfg(feature = "buf_overf_rem_all")]
                        assert_eq!(std::mem::size_of::<BufFile>(), 124);
                    }
                    #[cfg(target_arch = "arm")]
                    assert_eq!(std::mem::size_of::<BufFile>(), 136);
                }
                #[cfg(all(feature = "buf_stats", feature = "buf_lru"))]
                {
                    #[cfg(not(target_arch = "arm"))]
                    assert_eq!(std::mem::size_of::<BufFile>(), 88);
                    #[cfg(target_arch = "arm")]
                    assert_eq!(std::mem::size_of::<BufFile>(), 96);
                }
                #[cfg(all(feature = "buf_stats", not(feature = "buf_lru")))]
                {
                    #[cfg(not(target_arch = "arm"))]
                    assert_eq!(std::mem::size_of::<BufFile>(), 132);
                    #[cfg(target_arch = "arm")]
                    assert_eq!(std::mem::size_of::<BufFile>(), 144);
                }
                #[cfg(all(not(feature = "buf_stats"), feature = "buf_lru"))]
                {
                    #[cfg(not(target_arch = "arm"))]
                    {
                        #[cfg(not(feature = "buf_overf_rem_half"))]
                        assert_eq!(std::mem::size_of::<BufFile>(), 80);
                        #[cfg(feature = "buf_overf_rem_half")]
                        assert_eq!(std::mem::size_of::<BufFile>(), 92);
                    }
                    #[cfg(target_arch = "arm")]
                    {
                        #[cfg(not(feature = "buf_overf_rem_half"))]
                        assert_eq!(std::mem::size_of::<BufFile>(), 80);
                        #[cfg(feature = "buf_overf_rem_half")]
                        assert_eq!(std::mem::size_of::<BufFile>(), 104);
                    }
                }
            }
            //
            #[cfg(not(target_arch = "arm"))]
            {
                #[cfg(not(feature = "buf_overf_rem_all"))]
                assert_eq!(std::mem::size_of::<Chunk>(), 28);
                #[cfg(feature = "buf_overf_rem_all")]
                assert_eq!(std::mem::size_of::<Chunk>(), 24);
            }
            #[cfg(target_arch = "arm")]
            {
                #[cfg(not(feature = "buf_overf_rem_all"))]
                assert_eq!(std::mem::size_of::<Chunk>(), 32);
                #[cfg(feature = "buf_overf_rem_all")]
                assert_eq!(std::mem::size_of::<Chunk>(), 24);
            }
            //
            #[cfg(not(target_arch = "arm"))]
            assert_eq!(std::mem::size_of::<(u64, usize)>(), 12);
            #[cfg(target_arch = "arm")]
            assert_eq!(std::mem::size_of::<(u64, usize)>(), 16);
            //
            assert_eq!(std::mem::size_of::<Vec<Chunk>>(), 12);
            assert_eq!(std::mem::size_of::<Vec<u8>>(), 12);
        }
    }
}
