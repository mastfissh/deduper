#![allow(unused)]
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::fs::File;
use std::io::Write;

static LOREM_IPSUM: &str =
    "Lorem ipsum dolor sit amet, consectetur adipisicing elit, sed do eiusmod
tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam,
quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo
consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse
cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non
proident, sunt in culpa qui officia deserunt mollit anim id est laborum.
";

static PADDING: &str = "ab";

extern crate dupelib;

use std::fs;
use std::path::PathBuf;

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut sample_dir = PathBuf::from(file!());
    sample_dir.pop();
    sample_dir.push("samples");

    let mut lots_dir = sample_dir.clone();
    lots_dir.push("lots");
    fs::create_dir_all(&lots_dir).unwrap();
    let mut big_dir = sample_dir.clone();
    big_dir.push("big");
    fs::create_dir_all(&big_dir).unwrap();

    for x in 0..400 {
        let mut filename = lots_dir.clone();
        filename.push(x.to_string());
        let mut file = File::create(&filename).unwrap();
        file.write_all(LOREM_IPSUM.as_bytes());
        let extra = x % 10;
        for _ in 0..extra {
            file.write_all(PADDING.as_bytes());
        }
    }
    for x in 400..1000 {
        let mut filename = lots_dir.clone();
        filename.push(x.to_string());
        let mut file = File::create(&filename).unwrap();
        file.write_all(LOREM_IPSUM.as_bytes());
        for _ in 0..x {
            file.write_all(PADDING.as_bytes());
        }
    }
    fs::create_dir_all(&big_dir).unwrap();
    for x in 0..4 {
        let mut filename = big_dir.clone();
        filename.push(x.to_string());
        let mut file = File::create(&filename).unwrap();
        for _ in 0..100_000 {
            file.write_all(LOREM_IPSUM.as_bytes());
        }
    }
    let mut count = 0;
    for x in 4..8 {
        count += 1;
        let mut filename = big_dir.clone();
        filename.push(x.to_string());
        let mut file = File::create(&filename).unwrap();
        for _ in 0..100_000 {
            file.write_all(LOREM_IPSUM.as_bytes());
        }
        for _ in 0..count {
            file.write_all(PADDING.as_bytes());
        }
    }
    let mut count = 0;
    for x in 8..12 {
        count += 1;
        let mut filename = big_dir.clone();
        filename.push(x.to_string());
        let mut file = File::create(&filename).unwrap();
        for _ in 0..count {
            file.write_all(PADDING.as_bytes());
        }
        for _ in 0..100_000 {
            file.write_all(LOREM_IPSUM.as_bytes());
        }
    }
    c.bench_function("lots", |b| {
        b.iter(|| {
            let dir = lots_dir.clone();
            let options = dupelib::Opt {
                paths: vec![dir],
                ..Default::default()
            };
            dupelib::detect_dupes(options, None)
        })
    });

    c.bench_function("big", |b| {
        b.iter(|| {
            let dir = big_dir.clone();
            let options = dupelib::Opt {
                paths: vec![dir],
                ..Default::default()
            };
            dupelib::detect_dupes(options, None)
        })
    });

    c.bench_function("all", |b| {
        b.iter(|| {
            let dir = sample_dir.clone();
            let options = dupelib::Opt {
                paths: vec![dir],
                ..Default::default()
            };
            dupelib::detect_dupes(options, None)
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
