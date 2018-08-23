extern crate walkdir;
extern crate blake2;
extern crate rayon;
extern crate chashmap;
#[macro_use]
extern crate structopt;

use rayon::prelude::*;
use std::io::prelude::*;
use walkdir::DirEntry;
use std::io::Read;
use std::error::Error;
use blake2::digest::generic_array::GenericArray;
use blake2::digest::generic_array::typenum::U64;
use std::fs::File;
use walkdir::WalkDir;
use std::fs;
use std::path::PathBuf;
use blake2::{Blake2b, Digest};
use std::time::{Instant};
use chashmap::CHashMap;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
struct Opt {

  #[structopt(name = "paths", parse(from_os_str))]
  paths: Vec<PathBuf>,

  #[structopt(short = "t", long = "timing")]
  timing: bool,

  #[structopt(short = "d", long = "debug")]
  debug: bool,

  #[structopt(short = "o", long = "output", parse(from_os_str))]
  output: Option<PathBuf>,

}


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
  let options = Opt::from_args();
  let now = Instant::now();
  let paths = CHashMap::new();
  options.paths.par_iter().for_each(|path| {
    for entry in WalkDir::new(path).into_iter().filter_entry(|e| !is_hidden(e)).filter_map(|e| e.ok()) {
      paths.insert(PathBuf::from(entry.path()), ());
    }
  });

  if options.debug {
    println!("{} files found ", paths.len());
  }

  let def_dupes = CHashMap::new();
  let file_hashes = CHashMap::new();
  let temp: Vec<PathBuf> = paths.into_iter().map(|x| x.0).collect();
  temp.par_iter().for_each(|current_path| {
    if let Ok(data) = byte_count_file(PathBuf::from(&current_path)) {
      if let Some(path) = file_hashes.insert(data, current_path.clone()) {
        def_dupes.insert(current_path.clone(), ());
        def_dupes.insert(path.to_path_buf(), ());
      }
    }
  });
  let paths = def_dupes;


  if options.debug {
    println!("{} potential dupes after filesize cull", paths.len());
  }


  let def_dupes = CHashMap::new();
  let file_hashes = CHashMap::new();
  let temp: Vec<PathBuf> = paths.into_iter().map(|x| x.0).collect();
  temp.par_iter().for_each(|current_path| {
    if let Ok(data) = hash_first_file(PathBuf::from(&current_path)) {
      if let Some(path) = file_hashes.insert(data, current_path.clone()) {
        def_dupes.insert(current_path.clone(),());
        def_dupes.insert(path.to_path_buf(),());
      }
    }
  });
  let paths = def_dupes;

  if options.debug {
    println!("{} potential dupes after first 500 bytes cull", paths.len());
  }

  let file_hashes = CHashMap::new();
  let temp: Vec<PathBuf> = paths.into_iter().map(|x| x.0).collect();
  let temp: Vec<(PathBuf, PathBuf)> = temp.par_iter().filter_map(|current_path| {
    if let Ok(data) = hash_file(PathBuf::from(&current_path)) {
      if let Some(path) = file_hashes.insert(data, current_path.clone()) {
        return Some((current_path.clone(), path.to_path_buf()));
      }
    }
    None
  }).collect();

  if options.debug {
    println!("{} dupes after full file hashing", temp.len());
  }

  let output_string = temp.par_iter().map(|item| {
    let (dupe1, dupe2) = item;
    format!(" {} | {} \n", dupe1.display(), dupe2.display())
  }).reduce(String::new, |mut start, item| {
    start.push_str(&item);
    start
  });

  if let Some(path) = options.output{
    let mut f = File::create(path).unwrap();
    f.write(output_string.as_bytes()).unwrap();
  } else {
    println!("{}", output_string);
  }
  if options.timing {
    print_timing_info(now);
  }

}
