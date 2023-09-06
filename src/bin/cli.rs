extern crate clap;
extern crate dupelib;

use clap::Parser;

use dupelib::detect_dupes;
use dupelib::Opt;

fn run_dupe_detect(options: Opt) {
    let dupes = detect_dupes(options);
    println!("{:?}", dupes);
}

fn main() {
    let opt = Opt::parse();
    run_dupe_detect(opt);
}
