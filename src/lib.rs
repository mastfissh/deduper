extern crate bytesize;
extern crate clap;
extern crate rayon;
extern crate walkdir;
use clap::Parser;
use rayon::prelude::*;
use seahash::SeaHasher;
use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::hash::Hasher;
use std::io::BufReader;
use std::io::Read;
use std::path::PathBuf;
use std::time::Instant;
use walkdir::DirEntry;
use walkdir::WalkDir;

#[derive(Parser, Debug, Default)]
pub struct Opt {
    pub paths: Vec<PathBuf>,

    #[arg(short, long)]
    pub timing: bool,

    #[arg(short, long)]
    pub debug: bool,

    #[arg(short, long)]
    pub minimum: Option<u64>,
}

type BoxResult<T> = Result<T, Box<dyn Error>>;

// given a path, returns the filesize of the file at that path
fn byte_count_file(path: &DirEntry) -> BoxResult<u64> {
    let metadata = path.metadata()?;
    Ok(metadata.len())
}

// given a path, returns a hash of the bytes of the file at that path
fn hash_file(path: &DirEntry) -> BoxResult<u64> {
    let file = File::open(path.path())?;
    let mut hasher = SeaHasher::new();
    let mut reader = BufReader::new(file);

    let mut buffer = vec![0; 512000];

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.write(&buffer[..count]);
    }

    Ok(hasher.finish())
}

// given a path, returns a hash of the first 64k bytes of the file at that path
fn hash_start_file(path: &DirEntry) -> BoxResult<u64> {
    let file = File::open(path.path())?;
    let mut hasher = SeaHasher::new();
    let mut reader = BufReader::new(file);
    let mut buffer = [0; 64000];
    let count = reader.read(&mut buffer)?;
    hasher.write(&buffer[..count]);
    Ok(hasher.finish())
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}

fn is_valid_file(res: Result<DirEntry, walkdir::Error>) -> Option<DirEntry> {
    if let Ok(entry) = res {
        if entry.file_type().is_file() {
            return Some(entry);
        };
    }
    None
}

fn print_timing_info(now: Instant) {
    println!(
        "Time since start was {}.{} secs",
        now.elapsed().as_secs(),
        now.elapsed().subsec_millis()
    );
}

fn walk_dirs(input: Vec<PathBuf>) -> Vec<CandidateFile> {
    let vec: Vec<DirEntry> = input
        .par_iter()
        .map(|path| {
            WalkDir::new(path)
                .into_iter()
                .filter_entry(|e| !is_hidden(e))
                .filter_map(is_valid_file)
                .collect::<Vec<DirEntry>>()
        })
        .flatten()
        .collect();
    let mut paths = Vec::new();
    for entry in vec {
        let item = CandidateFile { path: entry };
        paths.push(item);
    }
    paths
}

fn cull_by_filesize(input: Vec<CandidateFile>, minimum: u64) -> Vec<CandidateFileWithSize> {
    let input: Vec<_> = input
        .par_iter()
        .cloned()
        .filter_map(|candidate| {
            let current_path = &candidate.path;
            if let Ok(bytes_count) = byte_count_file(&current_path) {
                if bytes_count >= minimum {
                    let res = CandidateFileWithSize {
                        path: candidate.path,
                        size: bytes_count,
                    };
                    return Some(res);
                }
            }
            None
        })
        .collect();

    let mut hashes = HashSet::new();
    let mut dupe_hashes = HashSet::new();
    for candidate in &input {
        if hashes.contains(&candidate.size) {
            dupe_hashes.insert(candidate.size);
        } else {
            hashes.insert(candidate.size);
        }
    }

    let mut out = Vec::new();
    for candidate in input {
        if dupe_hashes.contains(&candidate.size) {
            out.push(candidate)
        }
    }
    out
}

fn cull_by_start(input: Vec<CandidateFileWithSize>) -> Vec<CandidateFileWithSizeAndHash> {
    let input: Vec<_> = input
        .par_iter()
        .cloned()
        .filter_map(|candidate| {
            let current_path = &candidate.path;
            if let Ok(hash) = hash_start_file(current_path) {
                let res = CandidateFileWithSizeAndHash {
                    path: candidate.path,
                    size: candidate.size,
                    hash: hash,
                };
                return Some(res);
            }
            None
        })
        .collect();

    let mut hashes = HashSet::new();
    let mut dupe_hashes = HashSet::new();
    for candidate in &input {
        if hashes.contains(&candidate.hash) {
            dupe_hashes.insert(candidate.hash);
        } else {
            hashes.insert(candidate.hash);
        }
    }

    let mut out = Vec::new();
    for candidate in input {
        if dupe_hashes.contains(&candidate.hash) {
            out.push(candidate)
        }
    }
    out
}

fn cull_by_hash(input: Vec<CandidateFileWithSizeAndHash>) -> Vec<CandidateFileWithSizeAndHash> {
    let input: Vec<_> = input
        .par_iter()
        .cloned()
        .filter_map(|mut candidate| {
            let current_path = &candidate.path;
            if let Ok(hash) = hash_file(current_path) {
                candidate.hash = hash;
                return Some(candidate);
            }
            None
        })
        .collect();

    let mut hashes = HashSet::new();
    let mut dupe_hashes = HashSet::new();
    for candidate in &input {
        if hashes.contains(&candidate.hash) {
            dupe_hashes.insert(candidate.hash);
        } else {
            hashes.insert(candidate.hash);
        }
    }

    let mut out = Vec::new();
    for candidate in input {
        if dupe_hashes.contains(&candidate.hash) {
            out.push(candidate)
        }
    }
    out
}
use std::cmp::Ordering;

use bytesize::ByteSize;
fn format_results(mut input: Vec<CandidateFileWithSizeAndHash>) -> () {
    input.sort_unstable_by(|a, b| {
        let size_cmp = b.size.cmp(&a.size);
        if size_cmp != Ordering::Equal {
            return size_cmp;
        }
        let hash_cmp = b.hash.cmp(&a.hash);
        if hash_cmp != Ordering::Equal {
            return hash_cmp;
        }
        format!("{}", b.path.path().display()).cmp(&format!("{}", a.path.path().display()))
    });
    let mut last_size: u64 = 0;
    let mut last_hash: u64 = 0;
    for item in input {
        let hash = item.hash;
        if hash != last_hash {
            println!("-------");
            last_hash = hash;
        }
        let size = item.size;
        if size != last_size {
            println!("Size: {} ", ByteSize(size));
            last_size = size;
        }
        println!("Path: {} ", item.path.path().display());
    }
}

#[derive(Clone)]
struct CandidateFile {
    path: DirEntry,
}

#[derive(Clone)]
struct CandidateFileWithSize {
    path: DirEntry,
    size: u64,
}

#[derive(Clone)]
struct CandidateFileWithSizeAndHash {
    path: DirEntry,
    size: u64,
    hash: u64,
}

pub fn detect_dupes(options: Opt) -> () {
    let now = Instant::now();
    let paths = walk_dirs(options.paths);

    if options.debug {
        println!("{} files found ", paths.len());
    }

    let minimum = options.minimum.unwrap_or(1);

    let paths = cull_by_filesize(paths, minimum);

    if options.debug {
        println!("{} potential dupes after filesize cull", paths.len());
    }

    let paths = cull_by_start(paths);

    if options.debug {
        println!("{} potential dupes after start cull", paths.len());
    }

    let paths = cull_by_hash(paths);

    if options.debug {
        println!("{} dupes after full file hashing", paths.len());
    }

    let output_strings = format_results(paths);

    if options.timing {
        print_timing_info(now);
    }
    output_strings
}
