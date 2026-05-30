use rabuf::BufFile;

fn main() {
    let (buf_file_size, chunk_size) = BufFile::get_internal_sizes();
    println!("BufFile size: {} bytes", buf_file_size);
    println!("Chunk size: {} bytes", chunk_size);
}
