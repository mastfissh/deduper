extern crate walkdir;
extern crate blake2;

use blake2::digest::generic_array::GenericArray;
use blake2::digest::generic_array::typenum::U64;
use std::fs::File;
use std::path::Path;
use std::env;
use walkdir::WalkDir;
use std::collections::HashMap;
use blake2::{Blake2b, Digest};

fn hash_file(entry: walkdir::DirEntry) -> Option<GenericArray<u8, U64>> {
  let mut file = match File::open(entry.path()) {
    Err(why) => {
      print!("failed_open {}\n", why);
      return None
    }
    Ok(file) => file,
  };
  return Blake2b::digest_reader(&mut file).ok()
}

fn main() {
  let args: Vec<String> = env::args().collect();
  let path = Path::new(&args[1]);
  let mut file_hashes = HashMap::new();
  for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
    let thing = format!("{}", entry.path().display());
    match hash_file(entry) {
      None => {
      },
      Some(data) => {
        match file_hashes.get(&data) {
          Some(path) => println!("dupe {} | {}", thing, path),
          None => {}
        }
        file_hashes.insert(data, thing);
      }
    }
  }


}
