use super::{AutomatonImpl, HORIZON};
use crate::rule::Rule;
use rand::Rng;

pub struct Automaton {
    pub size: usize,
    pub states: u8,
    flop: bool,
    grid1: Vec<u8>,
    grid2: Vec<u8>,
    rule: Rule,
}

impl Automaton {
    pub fn new(states: u8, size: usize, rule: Rule) -> Automaton {
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
    pub fn single_update(&mut self, is: isize, js: isize) {
        let size = self.size;
        let mut ind: usize = 0;
        let mut pw = 0;
        let states = self.states as usize;
        let grid = self.grid();
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
        self.prev_grid()[is as usize * size + js as usize] = self.rule.get(ind);
    }

    #[inline]
    pub fn single_update_bound_check(&mut self, is: isize, js: isize) {
        let size = self.size;
        let mut ind: usize = 0;
        let mut pw = 0;
        let states = self.states as usize;
        let grid = self.grid();
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
        self.prev_grid()[is as usize * size + js as usize] = self.rule.get(ind);
    }

    #[inline]
    pub fn update(&mut self) {
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
}

impl AutomatonImpl for Automaton {
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
            .map(move |grid| duplicate_array(grid, size, scale)),
        )
    }

    fn get_size(&self) -> usize {
        self.size
    }

    fn get_states(&self) -> u8 {
        self.states
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

#[inline]
fn duplicate_array(s: Vec<u8>, size: usize, scale: u16) -> Vec<u8> {
    let scaled_size = size * scale as usize;
    let mut out = Vec::with_capacity(scaled_size * scaled_size);
    for i in 0..scaled_size {
        for j in 0..scaled_size {
            let item = s[(i / scale as usize) * size + (j / scale as usize)];
            out.push(item);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use crate::rule::Rule;
    use crate::Automaton;
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
