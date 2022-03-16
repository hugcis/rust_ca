//! CA library
#![feature(test)]
#![deny(missing_docs)]

extern crate test;

pub mod automaton;
pub mod output;
pub mod rule;


#[cfg(test)]
mod tests {
    use crate::test::Bencher;
    use crate::automaton::AutomatonImpl;
    use crate::automaton::{Automaton, TiledAutomaton};
    use crate::output;
    use crate::rule::Rule;

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
