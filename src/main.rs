extern crate walkdir;
extern crate blake2;

use std::error::Error;
use blake2::digest::generic_array::GenericArray;
use blake2::digest::generic_array::typenum::U64;
use std::fs::File;
use std::path::Path;
use std::env;
use walkdir::WalkDir;
use std::collections::HashMap;
use blake2::{Blake2b, Digest};

type BoxResult<T> = Result<T,Box<Error>>;

fn hash_file(entry: walkdir::DirEntry) -> BoxResult<GenericArray<u8, U64>> {
  let mut file = File::open(entry.path())?;
  return Ok(Blake2b::digest_reader(&mut file)?);
}

fn main() {
  let mut file_hashes = HashMap::new();
  for arg in env::args() {
    let path = Path::new(&arg);
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
      let thing = format!("{}", entry.path().display());
      if let Ok(data) = hash_file(entry) {
        if let Some(path) = file_hashes.get(&data) {
          println!("dupe {} | {}", thing, path);
        }
        file_hashes.insert(data, thing);
      }
    }
  }

}
