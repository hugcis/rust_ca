use rand::Rng;

use crate::rule::Rule;

pub struct Automaton<'a> {
    pub size: usize,
    pub states: u8,
    flop: bool,
    grid1: Vec<u8>,
    grid2: Vec<u8>,
    rule: &'a Rule,
}

impl<'a> Automaton<'a> {
    pub fn new(states: u8, size: usize, grid: Vec<u8>, rule: &Rule) -> Automaton {
        Automaton {
            states,
            size,
            flop: true,
            rule,
            grid1: grid.to_vec(),
            grid2: grid.to_vec(),
        }
    }

    pub fn random_init(&mut self) {
        let states = self.states;
        let mut rng = rand::thread_rng();
        for i in self.grid().iter_mut() {
            *i = rng.gen_range(0..states);
        }
    }

    #[inline]
    pub fn grid(&mut self) -> &mut Vec<u8> {
        if self.flop {
            &mut self.grid1
        } else {
            &mut self.grid2
        }
    }

    #[inline]
    pub fn prev_grid(&mut self) -> &mut Vec<u8> {
        if self.flop {
            &mut self.grid2
        } else {
            &mut self.grid1
        }
    }

    #[inline]
    pub fn single_update(&mut self, horizon: i8, is: isize, js: isize) {
        let size = self.size;
        let mut ind: usize = 0;
        let mut pw = 0;
        for a in -horizon..=horizon {
            for b in -horizon..=horizon {
                let idx =
                    ((is + isize::from(a)) * (size as isize) + (js + isize::from(b))) as usize;
                let current_val = self.grid()[idx] as usize;
                ind += (self.states as usize).pow(pw) * current_val;
                pw += 1;
            }
        }
        self.prev_grid()[is as usize * size + js as usize] = self.rule.get(ind);
    }

    #[inline]
    pub fn single_update_bound_check(&mut self, horizon: i8, is: isize, js: isize) {
        let size = self.size;
        let mut ind: usize = 0;
        let mut pw = 0;
        for a in -horizon..=horizon {
            for b in -horizon..=horizon {
                let idx = (((is + isize::from(a) + self.size as isize) % self.size as isize)
                    * (size as isize)
                    + (js + isize::from(b) + self.size as isize) % self.size as isize)
                    as usize;
                ind += (self.states as usize).pow(pw) * self.grid()[idx] as usize;
                pw += 1;
            }
        }
        self.prev_grid()[is as usize * size + js as usize] = self.rule.get(ind);
    }

    #[inline]
    pub fn update(&mut self) {
        let horizon = self.rule.horizon;
        let bounds_low = horizon as usize;
        let bounds_high = (self.size as isize - isize::from(horizon)) as usize;
        for i in bounds_low..bounds_high {
            for j in bounds_low..bounds_high {
                self.single_update(horizon, i as isize, j as isize)
            }
        }

        for j in 0..self.size {
            for i in 0..bounds_low {
                self.single_update_bound_check(horizon, i as isize, j as isize)
            }
            for i in bounds_high..self.size {
                self.single_update_bound_check(horizon, i as isize, j as isize)
            }
        }

        for i in bounds_low..bounds_high {
            for j in 0..bounds_low {
                self.single_update_bound_check(horizon, i as isize, j as isize)
            }
            for j in bounds_high..self.size {
                self.single_update_bound_check(horizon, i as isize, j as isize)
            }
        }

        self.flop = !self.flop;
    }

    pub fn skipped_iter(&'a mut self, steps: u32, skip: u32) -> AutomatonIterator<'a> {
        AutomatonIterator {
            autom: self,
            skip,
            steps,
            ct: 0,
        }
    }
}

pub struct AutomatonIterator<'a> {
    autom: &'a mut Automaton<'a>,
    skip: u32,
    steps: u32,
    ct: u32,
}

impl Iterator for AutomatonIterator<'_> {
    type Item = Vec<u8>;
    fn next(&mut self) -> Option<Vec<u8>> {
        if self.ct >= self.steps {
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
}
