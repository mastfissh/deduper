use structopt::StructOpt;

extern crate dupelib;

fn main() {
    let options = dupelib::Opt::from_args();
    dupelib::detect_dupes(options);
}
