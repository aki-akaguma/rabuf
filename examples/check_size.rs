use rabuf::{BufFile, Chunk};

fn main() {
    println!("BufFile size: {} bytes", std::mem::size_of::<BufFile>());
    println!("Chunk size: {} bytes", std::mem::size_of::<Chunk>());
}
