#![cfg_attr(test, deny(warnings))]
#![deny(missing_docs)]
//! The main crate for CellularAutomata-rs.


use clap::Clap;

use rust_ca::automaton::{Automaton, TiledAutomaton, TILE_SIZE};
use rust_ca::rule::Rule;
use rust_ca::output;
use rust_ca::rule;
use rust_ca::automaton::AutomatonImpl;

/// A CLI CA simulator. With no options, this runs a randomly sampled CA rule
/// with 2 states for 50 steps and outputs it as a gif file `test.gif`.
#[derive(Clap)]
#[clap(
    name = "Rust CA",
    version = "0.1.0",
    author = "Hugo Cisneros <hmj.cisneros@gmail.com>"
)]
struct Opts {
    /// The size of the 2D CA grid
    #[clap(short, long, default_value = "128")]
    size: u16,
    /// Number of states of the CA
    #[clap(short = 'n', long, default_value = "2")]
    states: u8,
    /// Simulation time
    #[clap(short = 't', long, default_value = "50")]
    steps: u32,
    /// Steps to skip at every time step for the output
    #[clap(short = 'k', long, default_value = "1")]
    skip: u32,
    #[clap(long, default_value = "1")]
    horizon: i8,
    #[clap(long, default_value = "10")]
    delay: u16,
    /// File to read a rule from. The file must contain a valid rule for the
    /// corresponding number of states
    #[clap(short, long)]
    file: Option<String>,
    #[clap(short, long)]
    pattern: Option<String>,
    #[clap(short, long, possible_values = &["uniform", "dirichlet"], default_value = "dirichlet")]
    rule_sampling: rule::SamplingMode,
    #[clap(long, default_value = "0")]
    rotate: u8,
}

struct SimulationOpts {
    size: u16,
    scale: u16,
    states: u8,
    _horizon: i8, // Hardcoded for now to 1
    steps: u32,
    skip: u32,
    delay: u16,
    rule: Rule,
    pattern: Option<String>,
    rotate: u8,
}

fn parse_opts(opts: Opts) -> SimulationOpts {
    let scale = if opts.size > 512 {
        2
    } else if opts.size > 256 {
        3
    } else {
        4
    };
    let rule = match opts.file {
        Some(fname) => Rule::from_file(&fname).unwrap(),
        None => match opts.rule_sampling {
            rule::SamplingMode::Dirichlet => {
                Rule::random_dirichlet(opts.horizon, opts.states, None)
            }
            rule::SamplingMode::Uniform => Rule::random(opts.horizon, opts.states),
        },
    };
    SimulationOpts {
        size: opts.size,
        scale,
        states: opts.states,
        _horizon: opts.horizon,
        steps: opts.steps,
        skip: opts.skip,
        rule,
        pattern: opts.pattern,
        delay: opts.delay,
        rotate: opts.rotate,
    }
}

fn main() {
    let opts = parse_opts(Opts::parse());
    if opts.size as usize % (TILE_SIZE - 1) == 0 {
        let mut a = TiledAutomaton::new(opts.states, opts.size.into(), opts.rule);
        a.random_init();
        if let Some(fname) = opts.pattern {
            a.init_from_pattern(&fname);
        }
        output::write_to_gif_file(
            Some("test.gif"),
            &mut a,
            opts.scale,
            opts.steps,
            opts.skip,
            opts.delay,
            opts.rotate,
        );
    } else {
        let mut a = Automaton::new(opts.states, opts.size.into(), opts.rule);
        a.random_init();
        if let Some(fname) = opts.pattern {
            a.init_from_pattern(&fname);
        }
        output::write_to_gif_file(
            Some("test.gif"),
            &mut a,
            opts.scale,
            opts.steps,
            opts.skip,
            opts.delay,
            opts.rotate,
        );
    };
}
