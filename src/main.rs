#[macro_use]
extern crate serde_derive;

pub mod paxos;

use paxos::{Simulation};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Bad input: {:?}", args);
    }

    let nworkers: usize = args[1]
        .parse()
        .expect(&format!("Failed to parse int from {}.", args[1]));

    let _ = Simulation::new(nworkers);
}
