//! This library gathers several tools to simulate and work with 2D Cellular
//! Automata (CA) grids.
//!
//! For example to simulate a randomly initialized 2-states CA of size 512 with
//! neighborhood size 1 for 32 steps:
//!
//! ```
//! use rust_ca::rule::Rule;
//! use rust_ca::automaton::{AutomatonImpl, Automaton};
//!
//! // We build a random CA rule with 2 states and neighborhood size 1,
//! let rule = Rule::random(1, 2);
//! // and a CA of size 128, with also two states.
//! let mut automaton = Automaton::new(2, 128, rule);
//! automaton.random_init();
//! // Simulate the CA for 32 steps.
//! automaton.iter(32);
//! ```
//!
//! We now write 10 steps (starting from the last generated one) of the CA to a
//! GIF animation.
//! ```
//! # use rust_ca::rule::Rule;
//! # use rust_ca::automaton::{AutomatonImpl, Automaton};
//! use rust_ca::output;
//!
//! # let rule = Rule::random(1, 2);
//! # let mut automaton = Automaton::new(2, 128, rule);
//! # automaton.random_init();
//! # automaton.iter(32);
//! output::write_to_gif_file(Some("test.gif"), &mut automaton, 1, 10, 1, 1, 0);
//! ```
//!
//! The `scale` parameters make the GIF larger by duplicating every pixel, and
//! the `skip` argument will only write a step every `skip` steps.
//! ```
//! # use rust_ca::rule::Rule;
//! # use rust_ca::automaton::{AutomatonImpl, Automaton};
//! # use rust_ca::output;
//! # let rule = Rule::random(1, 2);
//! # let mut automaton = Automaton::new(2, 128, rule);
//! # automaton.random_init();
//! output::write_to_gif_file(Some("test_bis.gif"), &mut automaton, 4, 100, 10, 1, 0);
//! ```
#![feature(test)]
#![deny(missing_docs)]

extern crate test;

pub mod automaton;
pub mod output;
pub mod rule;

#[cfg(test)]
mod tests {
    use crate::automaton::AutomatonImpl;
    use crate::automaton::{Automaton, TiledAutomaton};
    use crate::output;
    use crate::rule::Rule;
    use crate::test::Bencher;

    #[bench]
    fn write_autom(b: &mut Bencher) {
        let r = Rule::random(1, 3);
        let size: u16 = 512;
        let mut a = Automaton::new(3, size.into(), r);
        a.random_init();
        b.iter(|| output::write_to_gif_file(Some("test.gif"), &mut a, 1, 10, 1, 1, 0))
    }
    #[bench]
    fn write_autom_tiled(b: &mut Bencher) {
        let r = Rule::random(1, 3);
        let size: u16 = 512;
        let mut a = TiledAutomaton::new(3, size.into(), r);
        a.random_init();
        b.iter(|| output::write_to_gif_file(Some("test.gif"), &mut a, 1, 10, 1, 1, 0))
    }
}
