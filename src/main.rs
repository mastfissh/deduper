use structopt::StructOpt;

extern crate dupelib;

fn main() {
    let options = dupelib::Opt::from_args();
    let dupe_count = dupelib::detect_dupes(options);
    println!("{} dupes detected", dupe_count);
}
