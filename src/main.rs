extern crate walkdir;
extern crate blake2;

use std::error::Error;
use blake2::digest::generic_array::GenericArray;
use blake2::digest::generic_array::typenum::U64;
use std::fs::File;
use std::env;
use walkdir::WalkDir;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use blake2::{Blake2b, Digest};

type BoxResult<T> = Result<T,Box<Error>>;
type HashResult = GenericArray<u8, U64>;

fn hash_file(path: &Path) -> BoxResult<HashResult> {
  let mut file = File::open(path)?;
  return Ok(Blake2b::digest_reader(&mut file)?);
}

fn byte_count_file(path: &Path) -> BoxResult<u64> {
  let metadata = fs::metadata(path)?;
  return Ok(metadata.len());
}


fn main() {
  let mut dupes: HashSet<String> = HashSet::new();
  let mut file_sizes: HashMap<u64, String> = HashMap::new();
  for arg in env::args() {
    let path = Path::new(&arg);
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
      let current_path = format!("{}", entry.path().display());
      if let Ok(data) = byte_count_file(entry.path()) {
        if let Some(path) = file_sizes.get(&data) {
          dupes.insert(current_path.clone());
          dupes.insert(path.to_string());
        }
        file_sizes.insert(data, current_path.clone());
      }
    }
  }
  let mut def_dupes: HashSet<String> = HashSet::new();
  let mut file_hashes: HashMap<HashResult, String> = HashMap::new();
  for dupe in dupes {
    let current_path = format!("{}", dupe);
    if let Ok(data) = hash_file(Path::new(&dupe)) {
      if let Some(path) = file_hashes.get(&data) {
        def_dupes.insert(current_path.clone());
        def_dupes.insert(path.to_string());
      }
      file_hashes.insert(data, current_path.clone());
    }
  }
  for dupe in def_dupes {
    println!("dupe file: {}", dupe);
  }
}
