extern crate clap;
extern crate dupelib;

use clap::Parser;

use dupelib::detect_dupes;
use dupelib::Opt;

fn run_dupe_detect(options: Opt) {
    detect_dupes(options);
}

fn main() {
    let opt = Opt::parse();
    run_dupe_detect(opt);
}
