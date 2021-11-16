/*!
The Buffer for random access file.

When you read and write a file,
this read and write in `Chunk` units and reduce IO operation.

# Examples

## Write, Seek, Read

```rust
use rabuf::BufFile;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};

std::fs::create_dir_all("target/tmp").unwrap();

let path = "target/tmp/doc_test_1";
let bw = b"ABCEDFG\nhijklmn\n";

let f = File::create(path).unwrap();
let mut bf = BufFile::new(f).unwrap();
bf.write_all(bw).unwrap();

bf.seek(SeekFrom::Start(0)).unwrap();

let mut br = vec![0u8; bw.len()];
bf.read_exact(&mut br).unwrap();
assert_eq!(&br, bw);
```

## Write, Close, Open, Read

```rust
use rabuf::BufFile;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};

std::fs::create_dir_all("target/tmp").unwrap();
let path = "target/tmp/doc_test_2";

let bw = b"abcdefg\nHIJKLMN\n";
{
    let f = File::create(path).unwrap();
    let mut bf = BufFile::new(f).unwrap();
    bf.write_all(bw).unwrap();
}
{
    let f = File::open(path).unwrap();
    let mut bf = BufFile::new(f).unwrap();
    let mut br = vec![0u8; bw.len()];
    bf.read_exact(&mut br).unwrap();
    assert_eq!(&br, bw);
}
```
*/
use std::fs::File;
use std::io::{Read, Result, Seek, SeekFrom, Write};
//use std::io::{Error, ErrorKind};

/// Buffered File for ramdom access.
pub type BufFile = RaBuf<File>;

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
                let chunk = &self.chunks[i];
                if chunk.offset + chunk.data.len() as u64 >= size {
                    // data end is over the new end
                    // nothing todo
                } else if chunk.offset >= size {
                    // chunk start is over the new end
                    self.map.remove(&chunk.offset);
                    self.chunks[i].uses = 0;
                    self.fetch_cache = None;
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
            /*
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                format!(
                    "You tried to seek over the end of the file: {} < {}",
                    self.end, new_pos
                ),
            ));
            */
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
    fn sync_all(&mut self) -> Result<()> {
        self.flush()?;
        self.file.sync_all()
    }
    /// Flush buffer and call
    /// [`std::io::File.sync_data()`](https://doc.rust-lang.org/std/fs/struct.File.html#method.sync_data)
    fn sync_data(&mut self) -> Result<()> {
        self.flush()?;
        self.file.sync_data()
    }
}

/// Read small bytes less than chunk size.
pub trait SmallRead {
    /// Read one byte with a fast routine.
    fn read_one_byte(&mut self) -> Result<u8>;
    /// Read small size bytes with a fast routine. The small size is less than chunk size.
    fn read_exact_small(&mut self, buf: &mut [u8]) -> Result<()>;
}

impl SmallRead for BufFile {
    /// Read one byte with a fast routine.
    fn read_one_byte(&mut self) -> Result<u8> {
        let curr = self.pos;
        let one_byte = {
            let chunk = self.fetch_chunk(curr)?;
            let data_slice = &chunk.data[(curr - chunk.offset) as usize..];
            if !data_slice.is_empty() {
                data_slice[0]
            } else {
                let mut buf = [0u8; 1];
                let _ = self.read_exact(&mut buf)?;
                return Ok(buf[0]);
            }
        };
        self.pos += 1;
        Ok(one_byte)
    }
    /// Read small size bytes with a fast routine. The small size is less than chunk size.
    fn read_exact_small(&mut self, buf: &mut [u8]) -> Result<()> {
        debug_assert!(
            buf.len() <= self.chunk_size,
            "buf.len(): {} <= {}",
            buf.len(),
            self.chunk_size
        );
        let curr = self.pos;
        let len = {
            let chunk = self.fetch_chunk(curr)?;
            let buf_len = buf.len();
            let data_slice = &chunk.data[(curr - chunk.offset) as usize..];
            if buf_len <= data_slice.len() {
                buf.copy_from_slice(&data_slice[..buf_len]);
                buf_len
            } else {
                self.read_exact(buf)?;
                return Ok(());
            }
        };
        self.pos += len as u64;
        Ok(())
    }
}

/// Write small bytes less than chunk size.
pub trait SmallWrite {
    /// Write small size bytes with a fast routine. The small size is less than chunk size.
    fn write_all_small(&mut self, buf: &[u8]) -> Result<()>;
    /// Write zero of length `size` with a fast routine.
    fn write_zero(&mut self, size: u32) -> Result<()>;
}

impl SmallWrite for BufFile {
    /// Write small size bytes with a fast routine. The small size is less than chunk size.
    fn write_all_small(&mut self, buf: &[u8]) -> Result<()> {
        debug_assert!(
            buf.len() <= self.chunk_size,
            "buf.len(): {} <= {}",
            buf.len(),
            self.chunk_size
        );
        let curr = self.pos;
        let len = {
            let chunk = self.fetch_chunk(curr)?;
            let buf_len = buf.len();
            chunk.dirty = true;
            let data_slice = &mut chunk.data[(curr - chunk.offset) as usize..];
            if buf_len <= data_slice.len() {
                let dest = &mut data_slice[..buf_len];
                dest.copy_from_slice(buf);
                buf_len
            } else {
                return self.write_all(buf);
            }
        };
        self.pos += len as u64;
        if self.end < self.pos {
            self.end = self.pos;
        }
        Ok(())
    }
    /// Write zero of length `size` with a fast routine.
    fn write_zero(&mut self, size: u32) -> Result<()> {
        let size = size as usize;
        let curr = self.pos;
        let len = {
            let chunk = self.fetch_chunk(curr)?;
            chunk.dirty = true;
            let data_slice = &mut chunk.data[(curr - chunk.offset) as usize..];
            if size <= data_slice.len() {
                let dest = &mut data_slice[..size];
                dest.fill(0u8);
                size
            } else {
                let buf = vec![0u8; size];
                return self.write_all(&buf);
            }
        };
        self.pos += len as u64;
        if self.end < self.pos {
            self.end = self.pos;
        }
        Ok(())
    }
}

/// Chunk size MUST be a power of 2.
const CHUNK_SIZE: u32 = 1024 * 4;
const DEFAULT_NUM_CHUNKS: u16 = 16;

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
            let buf = if end_off >= chunk_size {
                &mut data[0..]
            } else {
                &mut data[0..end_off]
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
            uses: 0,
        })
    }
    //
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
            let buf = if end_off >= chunk_size {
                &mut data[0..]
            } else {
                &mut data[0..end_off]
            };
            if let Err(err) = file.read_exact(buf) {
                let _ = std::marker::PhantomData::<i32>;
                return Err(err);
            }
        }
        //
        self.dirty = false;
        self.uses = 0;
        self.offset = offset;
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
        let buf = if end_off >= self.data.len() {
            &self.data[0..]
        } else {
            &self.data[0..end_off]
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

/// Implements key-value sorted vec.
/// the key is the offset from start the file.
/// the value is the index of BufFile::data.
#[derive(Debug)]
struct OffsetIndex {
    vec: Vec<(u64, usize)>,
}
impl OffsetIndex {
    fn with_capacity(cap: usize) -> Self {
        Self {
            vec: Vec::with_capacity(cap),
        }
    }
    fn get(&mut self, offset: u64) -> Option<usize> {
        if let Ok(x) = self.vec.binary_search_by(|a| a.0.cmp(&offset)) {
            Some(self.vec[x].1)
        } else {
            None
        }
    }
    fn insert(&mut self, offset: u64, idx: usize) {
        match self.vec.binary_search_by(|a| a.0.cmp(&offset)) {
            Ok(x) => {
                self.vec[x].1 = idx;
            }
            Err(x) => {
                self.vec.insert(x, (offset, idx));
            }
        }
    }
    fn remove(&mut self, offset: &u64) -> Option<usize> {
        match self.vec.binary_search_by(|a| a.0.cmp(offset)) {
            Ok(x) => Some(self.vec.remove(x).1),
            Err(_x) => None,
        }
    }
    fn clear(&mut self) {
        self.vec.clear();
    }
}

/// Generic random access buffer.
#[derive(Debug)]
pub struct RaBuf<T: Seek + Read + Write> {
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
    // a minimum uses counter, but grater than 0.
    #[cfg(feature = "buf_stats")]
    stats_min_uses: u32,
    // a maximum uses counter
    #[cfg(feature = "buf_stats")]
    stats_max_uses: u32,
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
    pub fn new(file: File) -> Result<BufFile> {
        Self::with_capacity(file, DEFAULT_NUM_CHUNKS, CHUNK_SIZE)
    }
    /// Creates a new BufFile with the specified number of chunks.
    /// chunk_size is MUST power of 2.
    pub fn with_capacity(mut file: File, max_num_chunks: u16, chunk_size: u32) -> Result<BufFile> {
        debug_assert!(chunk_size == roundup_powerof2(chunk_size));
        let max_num_chunks = max_num_chunks as usize;
        let chunk_mask = !(chunk_size as u64 - 1);
        let chunk_size = chunk_size as usize;
        let end = file.seek(SeekFrom::End(0))?;
        file.seek(SeekFrom::Start(0))?;
        //
        Ok(BufFile {
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
        })
    }
    /// Flush and clear all buffer chunks.
    pub fn clear(&mut self) -> Result<()> {
        self.flush()?;
        self.fetch_cache = None;
        self.chunks.clear();
        self.map.clear();
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
    #[inline]
    fn touch(&mut self, chunk_idx: usize) {
        #[cfg(not(feature = "buf_lru"))]
        {
            self.chunks[chunk_idx].uses += 1;
        }
        #[cfg(feature = "buf_lru")]
        {
            self.uses_cnt += 1;
            self.chunks[chunk_idx].uses = self.uses_cnt;
        }
    }
    //
    fn fetch_chunk(&mut self, offset: u64) -> Result<&mut Chunk> {
        let offset = offset & self.chunk_mask;
        if let Some((off, idx)) = self.fetch_cache {
            if off == offset {
                self.touch(idx);
                return Ok(&mut self.chunks[idx]);
            }
        }
        let idx = if let Some(x) = self.map.get(offset) {
            x
        } else {
            self.add_chunk(offset)?
        };
        self.fetch_cache = Some((offset, idx));
        self.touch(idx);
        Ok(&mut self.chunks[idx])
    }
    //
    fn add_chunk(&mut self, offset: u64) -> Result<usize> {
        self.fetch_cache = None;
        if self.chunks.len() < self.max_num_chunks {
            let new_idx = self.chunks.len();
            match Chunk::new(offset, self.end, self.chunk_size, &mut self.file) {
                Ok(x) => {
                    self.map.insert(offset, new_idx);
                    self.chunks.push(x);
                    Ok(new_idx)
                }
                Err(e) => Err(e),
            }
        } else {
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
            Ok(min_idx)
        }
    }
}

impl<T: Seek + Read + Write> Read for RaBuf<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let curr = self.pos;
        let len = {
            let chunk = self.fetch_chunk(curr)?;
            let buf_len = buf.len();
            let mut data_slice = &chunk.data[(curr - chunk.offset) as usize..];
            if buf_len <= data_slice.len() {
                buf.copy_from_slice(&data_slice[..buf_len]);
                buf_len
            } else {
                data_slice.read(buf)?
            }
        };
        self.pos += len as u64;
        Ok(len)
    }
}

impl<T: Seek + Read + Write> Write for RaBuf<T> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let curr = self.pos;
        let len = {
            let chunk = self.fetch_chunk(curr)?;
            chunk.dirty = true;
            let mut data_slice = &mut chunk.data[(curr - chunk.offset) as usize..];
            data_slice.write(buf)?
        };
        self.pos += len as u64;
        if self.end < self.pos {
            self.end = self.pos;
        }
        Ok(len)
    }
    fn flush(&mut self) -> Result<()> {
        for chunk in self.chunks.iter_mut() {
            chunk.write(self.end, &mut self.file)?;
        }
        Ok(())
    }
}

/*
impl<T: Seek + Read + Write> Seek for RaBuf<T> {
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
            //self.set_len(new_pos)?;
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                format!(
                    "You tried to seek over the end of the file: {} < {}",
                    self.end, new_pos
                ),
            ));
        }
        self.pos = new_pos;
        Ok(new_pos)
    }
}
*/

impl<T: Seek + Read + Write> Drop for RaBuf<T> {
    /// Write all of the chunks to disk before closing the file.
    fn drop(&mut self) {
        let _ = self.flush();
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
            #[cfg(not(feature = "buf_stats"))]
            assert_eq!(std::mem::size_of::<BufFile>(), 120);
            #[cfg(feature = "buf_stats")]
            assert_eq!(std::mem::size_of::<BufFile>(), 128);
            //
            assert_eq!(std::mem::size_of::<Chunk>(), 40);
            assert_eq!(std::mem::size_of::<(u64, usize)>(), 16);
            assert_eq!(std::mem::size_of::<Vec<Chunk>>(), 24);
            assert_eq!(std::mem::size_of::<Vec<u8>>(), 24);
        }
        #[cfg(target_pointer_width = "32")]
        {
            #[cfg(not(any(feature = "buf_stats", feature = "buf_lru")))]
            {
                #[cfg(not(target_arch = "arm"))]
                assert_eq!(std::mem::size_of::<BufFile>(), 76);
                #[cfg(target_arch = "arm")]
                assert_eq!(std::mem::size_of::<BufFile>(), 88);
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
                assert_eq!(std::mem::size_of::<BufFile>(), 80);
                #[cfg(target_arch = "arm")]
                assert_eq!(std::mem::size_of::<BufFile>(), 88);
            }
            //
            #[cfg(not(target_arch = "arm"))]
            assert_eq!(std::mem::size_of::<Chunk>(), 28);
            #[cfg(target_arch = "arm")]
            assert_eq!(std::mem::size_of::<Chunk>(), 32);
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
