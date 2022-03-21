#![deny(missing_docs)]
use super::{AutomatonImpl, HORIZON};
use crate::automaton::duplicate_array;
use crate::{automaton::parse_pattern, rule::Rule};
use rand::Rng;
use std::ops::{Index, IndexMut};

/// The 2D Automaton object.
pub struct Automaton {
    /// The size of the 2D grid CA
    pub size: usize,
    /// The number of states available in each cell
    pub states: u8,
    flop: bool,
    grid1: Vec<u8>,
    grid2: Vec<u8>,
    rule: Rule,
}

impl Automaton {
    #[inline]
    fn prev_grid(&mut self) -> &mut Vec<u8> {
        if self.flop {
            &mut self.grid2
        } else {
            &mut self.grid1
        }
    }

    #[inline]
    /// Get a mutable reference to the current grid.
    pub fn grid_mut(&mut self) -> &mut Vec<u8> {
        if self.flop {
            &mut self.grid1
        } else {
            &mut self.grid2
        }
    }

    #[inline]
    fn single_update(&mut self, is: isize, js: isize) {
        let size = self.size;
        let mut ind: usize = 0;
        let mut pw = 0;
        let states = self.states as usize;
        let grid = self.grid_mut();
        for a in -HORIZON..=HORIZON {
            for b in -HORIZON..=HORIZON {
                let idx =
                    ((is + isize::from(a)) * (size as isize) + (js + isize::from(b))) as usize;
                let current_val = grid[idx] as usize;
                let power = states.pow(pw);
                ind += power * current_val;
                pw += 1;
            }
        }
        self.prev_grid()[is as usize * size + js as usize] = self.rule[ind];
    }

    #[inline]
    fn single_update_bound_check(&mut self, is: isize, js: isize) {
        let size = self.size;
        let mut ind: usize = 0;
        let mut pw = 0;
        let states = self.states as usize;
        let grid = self.grid_mut();
        for a in -HORIZON..=HORIZON {
            for b in -HORIZON..=HORIZON {
                let idx = (((is + isize::from(a) + size as isize) % size as isize)
                    * (size as isize)
                    + (js + isize::from(b) + size as isize) % size as isize)
                    as usize;
                let current_val = grid[idx] as usize;
                let power = states.pow(pw);
                ind += power * current_val;
                pw += 1;
            }
        }
        self.prev_grid()[is as usize * size + js as usize] = self.rule[ind];
    }
}

impl Index<usize> for Automaton {
    type Output = u8;
    fn index(&self, idx: usize) -> &Self::Output {
        if self.flop {
            &self.grid1[idx]
        } else {
            &self.grid2[idx]
        }
    }
}

impl IndexMut<usize> for Automaton {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        if self.flop {
            &mut self.grid1[idx]
        } else {
            &mut self.grid2[idx]
        }
    }
}


impl AutomatonImpl for Automaton {
    fn new(states: u8, size: usize, rule: Rule) -> Automaton {
        let grid = vec![0; size * size];
        Automaton {
            states,
            size,
            flop: true,
            rule,
            grid1: grid.to_vec(),
            grid2: grid.to_vec(),
        }
    }

    #[inline]
    fn grid(&self) -> &[u8] {
        if self.flop {
            &self.grid1
        } else {
            &self.grid2
        }
    }

    fn skipped_iter(
        &mut self,
        steps: u32,
        skip: u32,
        scale: u16,
    ) -> Box<dyn Iterator<Item = Vec<u8>> + '_> {
        let size = self.size;
        Box::new(
            AutomatonIterator {
                autom: self,
                skip,
                steps: Some(steps),
                ct: 0,
            }
            .map(move |grid| duplicate_array(&grid, size, scale)),
        )
    }

    fn size(&self) -> usize {
        self.size
    }

    fn states(&self) -> u8 {
        self.states
    }
    fn init_from_pattern(&mut self, pattern_fname: &str) {
        let pattern_spec = parse_pattern(pattern_fname).unwrap();
        assert!(pattern_spec.states <= self.states);
        assert!(pattern_spec.background < self.states);
        for i in self.grid_mut().iter_mut() {
            *i = pattern_spec.background;
        }
        let lines = pattern_spec.pattern.len();
        let cols = pattern_spec.pattern.iter().map(|x| x.len()).max().unwrap();
        for i in 0..lines {
            let lin = &pattern_spec.pattern[i];
            for (j, elem) in lin.iter().enumerate() {
                let idx =
                    (i + (self.size / 2) - lines / 2) * self.size + (j - cols / 2 + self.size / 2);
                self.grid_mut()[idx] = *elem;
            }
        }
    }

    #[inline]
    fn update(&mut self) {
        let bounds_low = HORIZON as usize;
        let bounds_high = (self.size as isize - isize::from(HORIZON)) as usize;
        //Main update
        for i in bounds_low..bounds_high {
            for j in bounds_low..bounds_high {
                self.single_update(i as isize, j as isize)
            }
        }

        //Bounds update
        for j in 0..self.size {
            for i in 0..bounds_low {
                self.single_update_bound_check(i as isize, j as isize)
            }
            for i in bounds_high..self.size {
                self.single_update_bound_check(i as isize, j as isize)
            }
        }

        for i in bounds_low..bounds_high {
            for j in 0..bounds_low {
                self.single_update_bound_check(i as isize, j as isize)
            }
            for j in bounds_high..self.size {
                self.single_update_bound_check(i as isize, j as isize)
            }
        }

        self.flop = !self.flop;
    }

    fn random_init(&mut self) {
        let states = self.states;
        let mut rng = rand::thread_rng();
        for i in self.grid_mut().iter_mut() {
            *i = rng.gen_range(0..states);
        }
    }
}

pub struct AutomatonIterator<'a> {
    autom: &'a mut Automaton,
    skip: u32,
    steps: Option<u32>,
    ct: u32,
}

impl Iterator for AutomatonIterator<'_> {
    type Item = Vec<u8>;
    fn next(&mut self) -> Option<Vec<u8>> {
        match self.steps {
            Some(v) => {
                if self.ct >= v {
                    None
                } else {
                    let ret = self.autom.grid().to_vec();
                    for _ in 0..self.skip {
                        self.autom.update();
                        self.ct += 1;
                    }
                    Some(ret)
                }
            }
            None => {
                let ret = self.autom.grid().to_vec();
                for _ in 0..self.skip {
                    self.autom.update();
                    self.ct += 1;
                }
                Some(ret)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::automaton::Automaton;
    use crate::automaton::AutomatonImpl;
    use crate::rule::Rule;
    use test::Bencher;

    fn get_random_auto(size: usize, states: u8) -> Automaton {
        let states = test::black_box(states);
        let rule = Rule::random(1, states);
        let mut a = Automaton::new(states, size, rule);
        a.random_init();
        a
    }

    #[test]
    fn update_should_apply_rule() {
        let mut a = get_random_auto(32, 2);
        let b1 = a.flop;
        a.update();
        assert_ne!(b1, a.flop);
    }

    #[bench]
    fn bench_update_one_item_bd(b: &mut Bencher) {
        let mut a = get_random_auto(64, 2);
        b.iter(|| a.single_update_bound_check(10, 10));
    }

    #[bench]
    fn bench_update_one_item(b: &mut Bencher) {
        let mut a = get_random_auto(64, 2);
        b.iter(|| a.single_update(10, 10));
    }

    #[bench]
    fn bench_single_update_32(b: &mut Bencher) {
        let mut a = get_random_auto(32, 3);
        b.iter(|| a.update());
    }

    #[bench]
    fn bench_single_update_64(b: &mut Bencher) {
        let mut a = get_random_auto(64, 3);
        b.iter(|| a.update());
    }

    #[bench]
    fn bench_single_update_128(b: &mut Bencher) {
        let mut a = get_random_auto(128, 3);
        b.iter(|| a.update());
    }

    #[bench]
    fn bench_single_update_256(b: &mut Bencher) {
        let mut a = get_random_auto(256, 3);
        b.iter(|| a.update());
    }

    #[bench]
    fn bench_single_update_512(b: &mut Bencher) {
        let mut a = get_random_auto(512, 3);
        b.iter(|| a.update());
    }
}
