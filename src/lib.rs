extern crate crossbeam_channel;
extern crate rayon;
extern crate structopt;
extern crate walkdir;

use crossbeam_channel::Sender;
use rayon::prelude::*;
use std::error::Error;
use std::fs::File;
use std::hash::Hasher;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::ops::Deref;
use std::ops::DerefMut;

use std::path::PathBuf;
use std::time::Instant;
use structopt::StructOpt;
use typed_arena::Arena;
use walkdir::DirEntry;
use walkdir::WalkDir;

use dashmap::DashMap;

struct HashableDirEntry(DirEntry);

impl Deref for HashableDirEntry {
    fn deref(&self) -> &Self::Target {
        &self.0
    }
    type Target = DirEntry;
}

impl DerefMut for HashableDirEntry {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::cmp::Eq for HashableDirEntry {}

impl std::cmp::PartialEq for HashableDirEntry {
    fn eq(&self, rhs: &HashableDirEntry) -> bool {
        self.path() == rhs.path()
    }
}

impl std::hash::Hash for HashableDirEntry {
    fn hash<H>(&self, h: &mut H)
    where
        H: std::hash::Hasher,
    {
        self.path().hash(h);
    }
}

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

impl Opt {
    pub fn from_paths(paths: Vec<PathBuf>) -> Opt {
        Opt {
            paths,
            timing: false,
            debug: false,
            output: None,
            minimum: None,
            sort: false,
        }
    }
}

type BoxResult<T> = Result<T, Box<dyn Error>>;

// given a path, returns the filesize of the file at that path
fn byte_count_file(path: &HashableDirEntry) -> BoxResult<u64> {
    let metadata = path.metadata()?;
    Ok(metadata.len())
}

use seahash::SeaHasher;

// given a path, returns a hash of all the bytes of the file at that path
fn hash_file(path: &HashableDirEntry) -> BoxResult<u64> {
    let file = File::open(path.path())?;
    let mut hasher = SeaHasher::new();
    let mut reader = BufReader::new(file);

    let mut buffer = [0; 64000];

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.write(&buffer[..count]);
    }

    Ok(hasher.finish())
}

// given a path, returns a hash of all the bytes of the file at that path
fn hash_start_file(path: &HashableDirEntry) -> BoxResult<u64> {
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

fn walk_dirs<'a>(
    input: Vec<PathBuf>,
    arena: &'a Arena<HashableDirEntry>,
) -> DashMap<&'a HashableDirEntry, ()> {
    let vec: Vec<DirEntry> = input
        .par_iter()
        .map(|path| {
            WalkDir::new(path)
                .into_iter()
                .filter_entry(|e| !is_hidden(e))
                .filter_map(|e| is_valid_file(e))
                .collect::<Vec<DirEntry>>()
        })
        .flatten()
        .collect();
    let paths = DashMap::new();
    for entry in vec {
        let item = arena.alloc(HashableDirEntry(entry));
        paths.insert(&*item, ());
    }
    paths
}

fn cull_by_filesize(
    input: DashMap<&HashableDirEntry, ()>,
    minimum: u64,
) -> DashMap<&HashableDirEntry, u64> {
    let dupes = DashMap::new();
    let file_hashes = DashMap::new();
    input
        .into_iter()
        .par_bridge()
        .for_each(|(current_path, _)| {
            if let Ok(bytes_count) = byte_count_file(current_path) {
                if bytes_count >= minimum {
                    if let Some(path) = file_hashes.insert(bytes_count, current_path) {
                        dupes.insert(current_path, bytes_count);
                        dupes.insert(path, bytes_count);
                    }
                }
            }
        });
    dupes
}

fn cull_by_start(input: DashMap<&HashableDirEntry, u64>) -> DashMap<&HashableDirEntry, u64> {
    let dupes = DashMap::new();
    let file_hashes = DashMap::new();
    input
        .into_iter()
        .par_bridge()
        .for_each(|(current_path, size)| {
            if size < 640_000 {
                dupes.insert(current_path, size);
            } else if let Ok(hash) = hash_start_file(current_path) {
                if let Some(path) = file_hashes.insert(hash, current_path) {
                    dupes.insert(current_path, size);
                    dupes.insert(path, size);
                }
            }
        });
    dupes
}

fn cull_by_hash(
    input: DashMap<&HashableDirEntry, u64>,
) -> Vec<(&HashableDirEntry, &HashableDirEntry, u64)> {
    let file_hashes = DashMap::new();
    input
        .into_iter()
        .par_bridge()
        .filter_map(|(current_path, bytes_count)| {
            if let Ok(hash) = hash_file(current_path) {
                if let Some(path) = file_hashes.insert(hash, current_path) {
                    return Some((current_path, path, bytes_count));
                }
            }
            None
        })
        .collect::<Vec<(_, _, _)>>()
}

fn format_results(input: &[(&HashableDirEntry, &HashableDirEntry, u64)]) -> Vec<String> {
    input
        .par_iter()
        .map(|item| {
            let (dupe1, dupe2, bytes_count) = item;
            format!(
                "{}: {} | {} \n",
                bytes_count,
                dupe1.path().display(),
                dupe2.path().display()
            )
        })
        .collect::<Vec<_>>()
}

fn maybe_send_progress<'a>(progress: &Option<Sender<&'a str>>, message: &'a str) {
    if let Some(p) = progress {
        p.send(message).unwrap();
    }
}

pub fn detect_dupes(options: Opt, progress: Option<Sender<&str>>) -> Vec<String> {
    let now = Instant::now();
    maybe_send_progress(&progress, "Walking dirs");
    let arena = Arena::new();
    let paths = walk_dirs(options.paths, &arena);

    if options.debug {
        println!("{} files found ", paths.len());
    }

    let minimum = options.minimum.unwrap_or(0);

    maybe_send_progress(&progress, "Culling by filesizes");
    let paths = cull_by_filesize(paths, minimum);

    if options.debug {
        println!("{} potential dupes after filesize cull", paths.len());
    }

    maybe_send_progress(&progress, "Culling by start");
    let paths = cull_by_start(paths);

    if options.debug {
        println!("{} potential dupes after start cull", paths.len());
    }

    maybe_send_progress(&progress, "Culling by hash");
    let mut confirmed_dupes = cull_by_hash(paths);

    if options.debug {
        println!("{} dupes after full file hashing", confirmed_dupes.len());
    }
    if options.sort {
        confirmed_dupes.sort_by_cached_key(|confirmed_dup| confirmed_dup.2);
    }
    maybe_send_progress(&progress, "Formatting");
    let output_strings = format_results(&confirmed_dupes);

    if let Some(path) = options.output {
        let mut f = File::create(path).unwrap();
        f.write_all(output_strings.join("").as_bytes()).unwrap();
    }
    if options.timing {
        print_timing_info(now);
    }
    output_strings
}
