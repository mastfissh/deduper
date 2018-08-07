extern crate walkdir;
extern crate blake2;
extern crate rayon;

use rayon::prelude::*;
use walkdir::DirEntry;
use std::io::Read;
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
use std::time::{Instant};

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

fn is_hidden(entry: &DirEntry) -> bool {
    entry.file_name()
         .to_str()
         .map(|s| s.starts_with("."))
         .unwrap_or(false)
}

fn print_timing_info(now: Instant){
  println!("Time since start was {}.{} secs", now.elapsed().as_secs(), now.elapsed().subsec_millis());
}

fn main() {
  let now = Instant::now();
  let paths = Arc::new(Mutex::new(HashSet::new()));
  let vec: Vec<_> = env::args().collect();
  vec.par_iter().for_each(|arg| {
    let path = PathBuf::from(&arg);
    for entry in WalkDir::new(path).into_iter().filter_entry(|e| !is_hidden(e)).filter_map(|e| e.ok()) {
      paths.lock().unwrap().insert(PathBuf::from(entry.path()));
    }
  });

  let def_dupes = Arc::new(Mutex::new(HashSet::new()));
  let file_hashes = Arc::new(Mutex::new(HashMap::new()));
  paths.lock().unwrap().par_iter().for_each(|current_path| {
    if let Ok(data) = byte_count_file(PathBuf::from(&current_path)) {
      if let Some(path) = file_hashes.lock().unwrap().insert(data, current_path.clone()) {
        def_dupes.lock().unwrap().insert(current_path.clone());
        def_dupes.lock().unwrap().insert(path.to_path_buf());
      }
    }
  });
  let paths = def_dupes;

  let def_dupes = Arc::new(Mutex::new(HashSet::new()));
  let file_hashes = Arc::new(Mutex::new(HashMap::new()));
  paths.lock().unwrap().par_iter().for_each(|current_path| {
    if let Ok(data) = hash_first_file(PathBuf::from(&current_path)) {
      if let Some(path) = file_hashes.lock().unwrap().insert(data, current_path.clone()) {
        def_dupes.lock().unwrap().insert(current_path.clone());
        def_dupes.lock().unwrap().insert(path.to_path_buf());
      }
    }
  });
  let paths = def_dupes;

  let file_hashes = Arc::new(Mutex::new(HashMap::new()));
  let out = Arc::new(Mutex::new(Vec::<(PathBuf, PathBuf)>::new()));
  paths.lock().unwrap().par_iter().for_each(|current_path| {
    if let Ok(data) = hash_file(PathBuf::from(&current_path)) {
      if let Some(path) = file_hashes.lock().unwrap().insert(data, current_path.clone()) {
        out.lock().unwrap().push((current_path.clone(), path.to_path_buf()));
      }
    }
  });

  for (dupe1, dupe2) in out.lock().unwrap().clone() {
    println!("dupe: {} | AND | {}", dupe1.display(), dupe2.display());
  }
  print_timing_info(now);

}
