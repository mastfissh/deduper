extern crate walkdir;
extern crate blake2;

use std::io::Read;
use std::error::Error;
use blake2::digest::generic_array::GenericArray;
use blake2::digest::generic_array::typenum::U64;
use std::fs::File;
use std::env;
use walkdir::WalkDir;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use blake2::{Blake2b, Digest};

type BoxResult<T> = Result<T,Box<Error>>;
type HashResult = GenericArray<u8, U64>;

fn byte_count_file(path: PathBuf) -> BoxResult<u64> {
  let metadata = fs::metadata(path)?;
  return Ok(metadata.len());
}

fn get_byte_count_identical<'a, I>(paths: I) -> HashSet<PathBuf>
where
    I: Iterator<Item = &'a PathBuf>,
{
  let mut dupes: HashSet<PathBuf> = HashSet::new();
  let mut file_sizes: HashMap<u64, PathBuf> = HashMap::new();
  for current_path in paths {
    if let Ok(data) = byte_count_file(PathBuf::from(&current_path)) {
      if let Some(path) = file_sizes.get(&data) {
        dupes.insert(current_path.clone());
        dupes.insert(path.to_path_buf());
      }
      file_sizes.insert(data, current_path.clone());
    }
  }
  return dupes
}

fn hash_file(path: PathBuf) -> BoxResult<HashResult> {
  let mut file = File::open(path)?;
  return Ok(Blake2b::digest_reader(&mut file)?);
}

fn hash_first_file(path: PathBuf) -> BoxResult<HashResult> {
  let mut file = File::open(path)?;
  let mut buffer = [0; 1000];
  file.read(&mut buffer[..])?;
  return Ok(Blake2b::digest_reader(&mut buffer.as_ref())?);
}

fn generic_check<'a, I>(check_fn: &Fn(PathBuf) -> BoxResult<HashResult>, paths: I) -> HashSet<PathBuf>
where
    I: Iterator<Item = &'a PathBuf>,
{
  let mut def_dupes: HashSet<PathBuf> = HashSet::new();
  let mut file_hashes: HashMap<HashResult, PathBuf> = HashMap::new();
  for current_path in paths {
    if let Ok(data) = check_fn(PathBuf::from(&current_path)) {
      if let Some(path) = file_hashes.get(&data) {
        def_dupes.insert(current_path.clone());
        def_dupes.insert(path.to_path_buf());
      }
      file_hashes.insert(data, current_path.clone());
    }
  }
  return def_dupes
}

fn main() {
  let mut paths: HashSet<PathBuf> = HashSet::new();
  for arg in env::args() {
    let path = PathBuf::from(&arg);
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
      paths.insert(PathBuf::from(entry.path()));
    }
  }
  let paths: HashSet<PathBuf> = get_byte_count_identical(paths.iter());
  let paths: HashSet<PathBuf> = generic_check(&hash_first_file, paths.iter());
  let paths: HashSet<PathBuf> = generic_check(&hash_file, paths.iter());

  for dupe in paths {
    println!("dupe file: {}", dupe.display());
  }
}
