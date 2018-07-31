extern crate walkdir;
extern crate blake2;
extern crate rayon;

use rayon::prelude::*;
use walkdir::DirEntry;
use std::io::Read;
use std::hash::Hash;
use std::sync::{Arc, Mutex};
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

fn generic_check<'a, I, T>(check_fn: fn(PathBuf) -> BoxResult<T>, paths: I) -> Arc<Mutex<HashSet<PathBuf>>>
where
    I: ParallelIterator<Item = &'a PathBuf>,
    T: Eq + Hash + Sync + Send
{
  let def_dupes = Arc::new(Mutex::new(HashSet::new()));
  let file_hashes: Arc<Mutex<HashMap<T, PathBuf>>> = Arc::new(Mutex::new(HashMap::new()));
  paths.for_each(|current_path| {
    if let Ok(data) = check_fn(PathBuf::from(&current_path)) {
      if let Some(path) = file_hashes.lock().unwrap().insert(data, current_path.clone()) {
        def_dupes.lock().unwrap().insert(current_path.clone());
        def_dupes.lock().unwrap().insert(path.to_path_buf());
      }
    }
  });
  def_dupes
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry.file_name()
         .to_str()
         .map(|s| s.starts_with("."))
         .unwrap_or(false)
}


fn main() {
  let paths = Arc::new(Mutex::new(HashSet::new()));
  let vec: Vec<_> = env::args().collect();
  vec.par_iter().for_each(|arg| {
    let path = PathBuf::from(&arg);
    for entry in WalkDir::new(path).into_iter().filter_entry(|e| !is_hidden(e)).filter_map(|e| e.ok()) {
      paths.lock().unwrap().insert(PathBuf::from(entry.path()));
    }
  });
  let paths = generic_check(byte_count_file, paths.lock().unwrap().par_iter());
  let paths = generic_check(hash_first_file, paths.lock().unwrap().par_iter());
  let paths = generic_check(hash_file, paths.lock().unwrap().par_iter());
  for dupe in paths.lock().unwrap().clone() {
    println!("dupe: {}", dupe.display());
  }

}
