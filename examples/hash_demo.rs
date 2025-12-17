use movs::hash::hash_file;
use std::path::Path;

fn main() {
    // Hash this source file itself!
    let path = Path::new("examples/hash_demo.rs");
    
    match hash_file(path) {
        Ok(hash) => {
            println!("SHA-256 hash of {:?}:", path);
            println!("{}", hash);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}