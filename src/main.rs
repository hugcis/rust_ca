#![feature(test)]
#![cfg_attr(test, deny(warnings))]
#![deny(missing_docs)]
//! The main crate for Iron.
//!
//! ## Overview
//!
//! Iron is a high level web framework built in and for Rust, built on
//! [hyper](https://github.com/hyperium/hyper). Iron is designed to take advantage
//! of Rust's greatest features - its excellent type system and principled
//! approach to ownership in both single threaded and multi threaded contexts.
//!
//! Iron is highly concurrent and can scale horizontally on more machines behind a
//! load balancer or by running more threads on a more powerful machine. Iron
//! avoids the bottlenecks encountered in highly concurrent code by avoiding shared
//! writes and locking in the core framework.

// Third-party packages
extern crate test;

use crate::automaton::{Automaton, TiledAutomaton, TILE_SIZE};
use crate::rule::Rule;
use std::env;

mod automaton;
mod output;
mod rule;

struct SimulationOpts {
    size: u16,
    scale: u16,
    states: u8,
    _horizon: i8, // Hardcoded for now to 1
    steps: u32,
    skip: u32,
    rule: Rule,
}

fn print_usage(program: &str, opts: getopts::Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

fn make_parser() -> getopts::Options {
    let mut opts = getopts::Options::new();
    opts.optopt("s", "size", "size of the automaton", "SIZE");
    opts.optopt(
        "n",
        "states",
        "number of states of the automatom",
        "N_STATES",
    );
    opts.optopt("r", "radius", "radius of the rule", "RADIUS");
    opts.optopt("t", "steps", "simulation time", "N_STEPS");
    opts.optopt("k", "skip", "steps to skip every timestep", "SKIP");
    opts.optopt("f", "file", "rule file", "FILE");
    opts.optflag("h", "help", "print this help menu");
    opts
}

fn parse_args(opts: getopts::Options) -> getopts::Matches {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            panic!(f.to_string())
        }
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        std::process::exit(0);
    }
    matches
}

fn parse_opts(matches: getopts::Matches) -> SimulationOpts {
    let size: u16 = matches
        .opt_get("s")
        .expect("Error parsing size parameter")
        .unwrap_or(256);
    let scale = if size > 512 {
        2
    } else if size > 256 {
        3
    } else {
        4
    };
    let states = matches
        .opt_get("n")
        .expect("Error parsing states parameter")
        .unwrap_or(2);
    let horizon = matches
        .opt_get("r")
        .expect("Error parsing radius parameter")
        .unwrap_or(2);
    let steps = matches
        .opt_get("t")
        .expect("Error parsing steps parameter")
        .unwrap_or(100);
    let skip = matches
        .opt_get("k")
        .expect("Error parsing skip parameter")
        .unwrap_or(1);
    let rule = match matches.opt_str("f") {
        Some(fname) => Rule::from_file(&fname).unwrap(),
        None => Rule::random_dirichlet(horizon, states, None),
    };
    SimulationOpts {
        size,
        scale,
        states,
        _horizon: horizon,
        steps,
        skip,
        rule,
    }
}

fn main() {
    let opts = parse_opts(parse_args(make_parser()));
    if opts.size as usize % (TILE_SIZE - 1) == 0 {
        let mut a = TiledAutomaton::new(opts.states, opts.size.into(), opts.rule);
        a.random_init();
        output::write_to_gif_file_tiled(
            Some("test.gif"),
            &mut a,
            opts.scale,
            opts.steps,
            opts.skip,
        );
    } else {
        let mut a = Automaton::new(
            opts.states,
            opts.size.into(),
            vec![0; opts.size as usize * opts.size as usize],
            opts.rule,
        );
        a.random_init();
        output::write_to_gif_file(Some("test.gif"), &mut a, opts.scale, opts.steps, opts.skip);
    };
}

#[cfg(test)]
mod tests {
    use super::test::Bencher;
    use crate::automaton::{Automaton, TiledAutomaton};
    use crate::output;
    use crate::rule::Rule;

    #[bench]
    fn write_autom(b: &mut Bencher) {
        let r = Rule::random(1, 3);
        let size: u16 = 512;
        let mut a = Automaton::new(3, size.into(), vec![0; size as usize * size as usize], r);
        a.random_init();
        b.iter(|| output::write_to_gif_file(Some("test.gif"), &mut a, 1, 10, 1))
    }
    #[bench]
    fn write_autom_tiled(b: &mut Bencher) {
        let r = Rule::random(1, 3);
        let size: u16 = 512;
        let mut a = TiledAutomaton::new(3, size.into(), r);
        a.random_init();
        b.iter(|| output::write_to_gif_file_tiled(Some("test.gif"), &mut a, 1, 10, 1))
    }
}
