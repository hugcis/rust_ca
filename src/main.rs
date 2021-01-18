use crate::automaton::Automaton;
use crate::rule::Rule;
use std::env;

mod automaton;
mod output;
mod rule;

fn print_usage(program: &str, opts: getopts::Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = getopts::Options::new();
    opts.optopt("s", "size", "size of the automaton", "SIZE");
    opts.optopt(
        "n",
        "states",
        "number of states of the automatom",
        "N_STATES",
    );
    opts.optopt("r", "radius", "radius of the rule", "RADIUS");
    opts.optopt("f", "file", "rule file", "FILE");
    opts.optflag("h", "help", "print this help menu");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            panic!(f.to_string())
        }
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

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
    let r = match matches.opt_str("f") {
        Some(fname) => Rule::from_file(&fname).unwrap(),
        None => Rule::random(horizon, states),
    };
    let mut a = Automaton::new(
        states,
        size.into(),
        vec![0; size as usize * size as usize],
        &r,
    );
    a.random_init();
    output::write_to_gif_file(Some("test.gif"), &mut a, scale);
}
