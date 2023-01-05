#![cfg_attr(test, deny(warnings))]
#![deny(missing_docs)]
//! The main crate for rust_ca.

use core::panic;
use std::path::Path;

use clap::{ArgGroup, Parser};

use rust_ca::automaton::AutomatonImpl;
use rust_ca::automaton::{Automaton, TiledAutomaton, TILE_SIZE};
use rust_ca::output;
use rust_ca::rule::Rule;
use rust_ca::rule::{self, SamplingMode};

/// A CLI CA simulator. With no options, this runs a randomly sampled CA rule
/// with 2 states for 50 steps and outputs it as a gif file `test.gif`.
#[derive(Parser, Debug)]
#[clap(
    name = "Rust CA",
    version = "0.2.2",
    author = "Hugo Cisneros <hmj.cisneros@gmail.com>"
)]
#[clap(group(
            ArgGroup::new("write_rule")
                .required(false)
                .args(&["write_rule", "write_to_id"]),
        ))]
struct CLIOpts {
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
    /// File to read a rule from. The file must contain a valid rule
    /// for the corresponding number of states.
    #[clap(short, long)]
    file: Option<String>,
    /// File to write the rule to.
    #[clap(short, long)]
    write_rule: Option<String>,
    /// Write the rule to a file $ID.rule
    #[clap(long)]
    write_to_id: bool,
    /// Specify one of the implemented CA rule.
    #[clap(short, long, possible_values = &["GOL"])]
    rule: Option<String>,
    #[clap(short, long)]
    pattern: Option<String>,
    #[clap(long, possible_values = &["uniform", "dirichlet"], default_value = "dirichlet")]
    rule_sampling: rule::SamplingMode,
    #[clap(long, default_value = "0")]
    rotate: u8,
    /// Use a tiled CA (defaults to true when the size is a multiple of TILE_SIZE).
    #[clap(long)]
    use_tiled: bool,
    /// Make the rule symmetric (this will also apply to rules passed as files).
    #[clap(long)]
    symmetric: bool,
    /// A file to write the GIF to. Defaults to standard output.
    #[clap(short, long)]
    output: Option<String>,
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
    output: Option<String>,
}

impl SimulationOpts {
    /// Parse options from clap and construct a SimulationOpts object.
    fn from_clap_opts(opts: CLIOpts) -> Result<SimulationOpts, std::io::Error> {
        let scale = if opts.size > 512 {
            2
        } else if opts.size > 256 {
            3
        } else {
            4
        };
        let mut rule = if let Some(rule_name) = opts.rule {
            match rule_name.as_str() {
                "GOL" => Rule::gol(),
                _ => panic!("Unknown rule name"),
            }
        } else {
            let write_rule = if opts.write_to_id {
                RuleWrite::WriteToID
            } else {
                opts.write_rule
                    .as_ref()
                    .map_or(RuleWrite::None, |s| RuleWrite::WriteToFile(s.to_string()))
            };
            match (opts.file, write_rule) {
                (Some(file), RuleWrite::WriteToID) => {
                    let r = Rule::from_file(&file).unwrap();
                    r.to_file(format!("{}.rule", r.id()))?;
                    r
                }
                (Some(file), RuleWrite::WriteToFile(s)) => {
                    let r = Rule::from_file(&file).unwrap();
                    r.to_file(s)?;
                    r
                }
                (Some(file), RuleWrite::None) => Rule::from_file(&file).unwrap(),
                (None, RuleWrite::WriteToFile(write)) => {
                    make_new_rule(opts.rule_sampling, opts.horizon, opts.states, Some(write))?
                }
                (None, RuleWrite::None) => {
                    make_new_rule::<String>(opts.rule_sampling, opts.horizon, opts.states, None)?
                }
                (None, RuleWrite::WriteToID) => {
                    let rule = make_new_rule::<String>(
                        opts.rule_sampling,
                        opts.horizon,
                        opts.states,
                        None,
                    )?;
                    rule.to_file(format!("{}.rule", rule.id()))?;
                    rule
                }
            }
        };
        if opts.symmetric {
            rule.symmetrize();
        }
        Ok(SimulationOpts {
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
            output: opts.output,
        })
    }
}

fn make_new_rule<P: AsRef<Path>>(
    sampling_mode: SamplingMode,
    horizon: i8,
    states: u8,
    path: Option<P>,
) -> Result<Rule, std::io::Error> {
    let rule = match sampling_mode {
        rule::SamplingMode::Dirichlet => Rule::random_dirichlet(horizon, states, None),
        rule::SamplingMode::Uniform => Rule::random(horizon, states),
    };

    if let Some(path) = path {
        rule.to_file(path)?;
    }
    Ok(rule)
}

enum RuleWrite {
    WriteToFile(String),
    None,
    WriteToID,
}

/// Generate a gif file from a automaton implementing AutomatonImpl. Will use
/// the options defined in `opts`.
fn generate_gif_from_init<T: AutomatonImpl>(a: &mut T, opts: &SimulationOpts) {
    if let Some(fname) = &opts.pattern {
        a.init_from_pattern(fname).unwrap();
    } else {
        a.random_init();
    }
    output::write_to_gif_file(
        opts.output.as_ref(),
        a,
        opts.scale,
        opts.steps,
        opts.skip,
        opts.delay,
        opts.rotate,
    )
    .expect("Error writing output");
}

/// Main CLI entrypoint.
fn main() {
    let opts: SimulationOpts = SimulationOpts::from_clap_opts(CLIOpts::parse()).unwrap();
    // If the size of the CA is a multiple of the TILE_SIZE, use the tiled
    // implementation.
    if opts.size as usize % (TILE_SIZE - 1) == 0 {
        generate_gif_from_init(
            &mut TiledAutomaton::new(opts.states, opts.size.into(), opts.rule.clone()),
            &opts,
        );
    }
    // Otherwise use the default implementation.
    else {
        generate_gif_from_init(
            &mut Automaton::new(opts.states, opts.size.into(), opts.rule.clone()),
            &opts,
        );
    };
}
