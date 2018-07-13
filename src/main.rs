extern crate walkdir;
extern crate blake2;

use walkdir::DirEntry;
use std::io::Read;
use std::hash::Hash;
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
  Ok(metadata.len())
}

fn hash_file(path: PathBuf) -> BoxResult<HashResult> {
  let mut file = File::open(path)?;
  Ok(Blake2b::digest_reader(&mut file)?)
}

fn hash_first_file(path: PathBuf) -> BoxResult<HashResult> {
  let mut file = File::open(path)?;
  let mut buffer = [0; 1000];
  file.read(&mut buffer[..])?;
  Ok(Blake2b::digest_reader(&mut buffer.as_ref())?)
}

fn generic_check<'a, I, T>(check_fn: &Fn(PathBuf) -> BoxResult<T>, paths: I) -> HashSet<PathBuf>
where
    I: Iterator<Item = &'a PathBuf>,
    T: Eq + Hash
{
  let mut def_dupes = HashSet::new();
  let mut file_hashes: HashMap<T, PathBuf> = HashMap::new();
  for current_path in paths {
    if let Ok(data) = check_fn(PathBuf::from(&current_path)) {
      if let Some(path) = file_hashes.get(&data) {
        def_dupes.insert(current_path.clone());
        def_dupes.insert(path.to_path_buf());
      }
      file_hashes.insert(data, current_path.clone());
    }
  }
  def_dupes
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry.file_name()
         .to_str()
         .map(|s| s.starts_with("."))
         .unwrap_or(false)
}


fn main() {
  let mut paths = HashSet::new();
  for arg in env::args() {
    let path = PathBuf::from(&arg);
    for entry in WalkDir::new(path).into_iter().filter_entry(|e| !is_hidden(e)).filter_map(|e| e.ok()) {
      paths.insert(PathBuf::from(entry.path()));
    }
  }
  let paths = generic_check(&byte_count_file, paths.iter());
  let paths = generic_check(&hash_first_file, paths.iter());
  let paths = generic_check(&hash_file, paths.iter());

  for dupe in paths {
    println!("dupe file: {}", dupe.display());
  }
}
