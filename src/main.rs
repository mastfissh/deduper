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
  let args: Vec<String> = env::args().collect();
  let path = Path::new(&args[1]);
  let mut file_hashes = HashMap::new();
  for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
    let thing = format!("{}", entry.path().display());

    match hash_file(entry) {
      Err(_err) => {
      },
      Ok(data) => {
        match file_hashes.get(&data) {
          Some(path) => println!("dupe {} | {}", thing, path),
          None => {}
        }
        file_hashes.insert(data, thing);
      }
    }
  }


}
