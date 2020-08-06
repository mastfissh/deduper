extern crate crossbeam_channel;
extern crate dupelib;

use crossbeam_channel::unbounded;
use crossbeam_channel::Receiver;
use crossbeam_channel::RecvError;
use crossbeam_channel::Sender;

use dupelib::detect_dupes;
use dupelib::Opt;

use std::thread;

use structopt::StructOpt;

fn run_dupe_detect(options: Opt) {
    let (sender, receiver): (Sender<&str>, Receiver<&str>) = unbounded();
    thread::spawn(move || {
        let mut cont = true;
        while cont {
            cont = false;
            let data = receiver.recv();
            if data != Err(RecvError) {
                cont = true;
                println!("{}", data.unwrap().to_string());
            }
        }
    });
    let dupes = detect_dupes(options, Some(sender));
    println!("{:?}", dupes);
}

fn main() {
    let opt = Opt::from_args();
    run_dupe_detect(opt);
}
