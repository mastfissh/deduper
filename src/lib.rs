extern crate blake2;
extern crate chashmap;
extern crate rayon;
extern crate structopt;
extern crate walkdir;
use blake2::digest::generic_array::typenum::U64;
use blake2::digest::generic_array::GenericArray;
use blake2::{Blake2b, Digest};
use chashmap::CHashMap;
use rayon::prelude::*;
use std::error::Error;
use std::{fs, io};
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::time::Instant;
use structopt::StructOpt;
use walkdir::DirEntry;
use walkdir::WalkDir;

#[derive(StructOpt, Debug, Default)]
pub struct Opt {
    #[structopt(name = "paths", parse(from_os_str))]
    pub paths: Vec<PathBuf>,

    #[structopt(short = "t", long = "timing")]
    pub timing: bool,

    #[structopt(short = "d", long = "debug")]
    pub debug: bool,

    #[structopt(short = "o", long = "output", parse(from_os_str))]
    pub output: Option<PathBuf>,

    #[structopt(short = "m", long = "minimum")]
    pub minimum: Option<u64>,

    #[structopt(short = "s", long = "sort")]
    pub sort: bool,
}

type BoxResult<T> = Result<T, Box<dyn Error>>;
type HashResult = GenericArray<u8, U64>;

// given a path, returns the filesize of the file at that path
fn byte_count_file(path: PathBuf) -> BoxResult<u64> {
    let metadata = fs::metadata(path)?;
    Ok(metadata.len())
}

// given a path, returns a hash of all the bytes of the file at that path
fn hash_file(path: PathBuf) -> BoxResult<HashResult> {
    let mut file = File::open(path)?;
    let mut hasher = Blake2b::new();
    io::copy(&mut file, &mut hasher)?;
    Ok(hasher.result())
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}

fn print_timing_info(now: Instant) {
    println!(
        "Time since start was {}.{} secs",
        now.elapsed().as_secs(),
        now.elapsed().subsec_millis()
    );
}

fn walk_dirs(input: Vec<PathBuf>) -> CHashMap<PathBuf, ()> {
    let paths = CHashMap::new();
    input.par_iter().for_each(|path| {
        for entry in WalkDir::new(path)
            .into_iter()
            .filter_entry(|e| !is_hidden(e))
            .filter_map(|e| e.ok())
        {
            paths.insert(PathBuf::from(entry.path()), ());
        }
    });
    return paths;
}

fn cull_by_filesize(input: CHashMap<PathBuf, ()>, minimum: u64) -> CHashMap<PathBuf, u64> {
    let dupes = CHashMap::new();
    let file_hashes = CHashMap::new();
    input
        .into_iter()
        .par_bridge()
        .for_each(|(current_path, _)| {
            if let Ok(bytes_count) = byte_count_file(PathBuf::from(&current_path)) {
                if bytes_count >= minimum {
                    if let Some(path) = file_hashes.insert(bytes_count, current_path.clone()) {
                        dupes.insert(current_path.clone(), bytes_count);
                        dupes.insert(path.to_path_buf(), bytes_count);
                    }
                }
            }
        });
    return dupes;
}

fn cull_by_hash(input: CHashMap<PathBuf, u64>) -> Vec<(PathBuf, PathBuf)> {
    let file_hashes = CHashMap::new();
    return input
        .into_iter()
        .par_bridge()
        .filter_map(|(current_path, _)| {
            if let Ok(data) = hash_file(PathBuf::from(&current_path)) {
                if let Some(path) = file_hashes.insert(data, current_path.clone()) {
                    return Some((current_path.clone(), path.to_path_buf()));
                }
            }
            None
        })
        .collect::<Vec<(_, _)>>();
}

fn format_results(mut input: Vec<(PathBuf, PathBuf)>, paths: CHashMap<PathBuf, u64>) -> String {
    input.sort_by_cached_key(|item| {
        paths.get(&item.0);
    });
    input
        .par_iter()
        .map(|item| {
            let (dupe1, dupe2) = item;
            format!(" {} | {} \n", dupe1.display(), dupe2.display())
        })
    .reduce(String::new, |mut start, item| {
        start.push_str(&item);
        start
    })
    }

    pub fn detect_dupes(options: Opt) -> usize {
        let now = Instant::now();
        let paths = walk_dirs(options.paths);
        if options.debug {
            println!("{} files found ", paths.len());
        }

        let minimum = options.minimum.unwrap_or(0);

        let paths = cull_by_filesize(paths, minimum);

        if options.debug {
            println!("{} potential dupes after filesize cull", paths.len());
        }

        let paths2 = paths.clone();
        let confirmed_dupes = cull_by_hash(paths);

        if options.debug {
            println!("{} dupes after full file hashing", confirmed_dupes.len());
        }

        let output_string = format_results(confirmed_dupes.clone(), paths2);

        if let Some(path) = options.output {
            let mut f = File::create(path).unwrap();
            f.write_all(output_string.as_bytes()).unwrap();
        } else {
            println!("{}", output_string);
        }
        if options.timing {
            print_timing_info(now);
        }
        return confirmed_dupes.len();
}
