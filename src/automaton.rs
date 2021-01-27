extern crate test;
use crate::rule::Rule;
use rand::Rng;

const HORIZON: i8 = 1;
pub const TILE_SIZE: usize = 257;

pub type TiledGrid = Vec<[u8; TILE_SIZE * TILE_SIZE]>;

pub struct TiledAutomaton {
    pub size: usize,
    pub n_tiles: usize,
    pub states: u8,
    flop: bool,
    grid1: TiledGrid,
    grid2: TiledGrid,
    rule: Rule,
}

impl TiledAutomaton {
    pub fn new(states: u8, size: usize, rule: Rule) -> TiledAutomaton {
        let s = size / (TILE_SIZE - 1);
        TiledAutomaton {
            states,
            n_tiles: s,
            size,
            flop: true,
            rule,
            grid1: vec![[0; TILE_SIZE * TILE_SIZE]; s * s],
            grid2: vec![[0; TILE_SIZE * TILE_SIZE]; s * s],
        }
    }

    pub fn random_init(&mut self) {
        let states = self.states;
        let mut rng = rand::thread_rng();
        for i in self.grid().iter_mut() {
            for j in i.iter_mut() {
                *j = rng.gen_range(0..states);
            }
        }
    }

    #[inline]
    pub fn grid(&mut self) -> &mut TiledGrid {
        if self.flop {
            &mut self.grid1
        } else {
            &mut self.grid2
        }
    }

    #[inline]
    pub fn prev_grid(&mut self) -> &mut TiledGrid {
        if self.flop {
            &mut self.grid2
        } else {
            &mut self.grid1
        }
    }

    #[inline]
    pub fn update_tile(&mut self, tx: usize, ty: usize) {
        let n_tiles = self.n_tiles;
        let states = self.states as usize;
        let grid = self.grid()[tx * n_tiles + ty];
        let mut prev_grid = self.prev_grid()[tx * n_tiles + ty];
        for i in HORIZON as usize..TILE_SIZE - HORIZON as usize {
            for j in HORIZON as usize..TILE_SIZE - HORIZON as usize {
                let is = i as isize;
                let js = j as isize;
                let mut ind: usize = 0;
                let mut pw = 0;
                for a in -HORIZON..=HORIZON {
                    for b in -HORIZON..=HORIZON {
                        let idx = ((is + isize::from(a)) * (TILE_SIZE as isize)
                            + (js + isize::from(b))) as usize;
                        let current_val = grid[idx] as usize;
                        let power = states.pow(pw);
                        ind += power * current_val;
                        pw += 1;
                    }
                }
                prev_grid[i * TILE_SIZE + j] = self.rule.get(ind);
            }
        }
    }

    #[inline]
    pub fn update_tile_boundaries(&mut self, tx: usize, ty: usize) {
        let states = self.states as usize;
        let n_tiles = self.n_tiles;
        let prev_x = (tx - 1 + self.n_tiles) % self.n_tiles;
        let prev_y = (ty - 1 + self.n_tiles) % self.n_tiles;
        let lmain_tile = self.grid()[tx * n_tiles + ty];
        let lnorth_tile = self.grid()[prev_x * n_tiles + ty];
        let lwest_tile = self.grid()[tx * n_tiles + prev_y];
        let lnorthwest_tile = self.grid()[prev_x * n_tiles + prev_y];
        let mut main_tile = self.prev_grid()[tx * n_tiles + ty];
        let mut north_tile = self.prev_grid()[prev_x * n_tiles + ty];
        let mut west_tile = self.prev_grid()[tx * n_tiles + prev_y];
        let mut northwest_tile = self.prev_grid()[prev_x * n_tiles + prev_y];

        for i in 1..TILE_SIZE - 1 {
            let is = i as isize;
            let mut ind: usize = 0;
            let mut pw = 0;
            for a in -HORIZON..=HORIZON {
                for b in -HORIZON..=HORIZON {
                    let current_val = if b < 0 {
                        let idx = ((is + isize::from(a)) * (TILE_SIZE as isize)
                            + (TILE_SIZE as isize - 1 + isize::from(b)))
                            as usize;
                        lwest_tile[idx] as usize
                    } else {
                        let idx = ((is + isize::from(a)) * (TILE_SIZE as isize) + isize::from(b))
                            as usize;
                        lmain_tile[idx] as usize
                    };
                    let power = states.pow(pw);
                    ind += power * current_val;
                    pw += 1;
                }
            }
            main_tile[i * TILE_SIZE] = self.rule.get(ind);
            west_tile[i * TILE_SIZE + (TILE_SIZE - 1)] = self.rule.get(ind);
        }
        for j in 1..TILE_SIZE - 1 {
            let js = j as isize;
            let mut ind: usize = 0;
            let mut pw = 0;
            for a in -HORIZON..=HORIZON {
                for b in -HORIZON..=HORIZON {
                    let current_val = if a < 0 {
                        let idx = ((TILE_SIZE as isize - 1 + isize::from(a)) * (TILE_SIZE as isize)
                            + (js + isize::from(b))) as usize;
                        lnorth_tile[idx] as usize
                    } else {
                        let idx = (js + isize::from(b)) as usize;
                        lmain_tile[idx] as usize
                    };
                    let power = states.pow(pw);
                    ind += power * current_val;
                    pw += 1;
                }
            }
            main_tile[j] = self.rule.get(ind);
            north_tile[(TILE_SIZE - 1) * TILE_SIZE + j] = self.rule.get(ind);
        }

        let mut ind: usize = 0;
        let mut pw = 0;
        for a in -HORIZON..=HORIZON {
            for b in -HORIZON..=HORIZON {
                let current_val = if (a < 0) & (b < 0) {
                    let idx = ((TILE_SIZE as isize - 1 + isize::from(a)) * (TILE_SIZE as isize)
                        + (TILE_SIZE as isize - 1 + isize::from(b)))
                        as usize;
                    lnorthwest_tile[idx] as usize
                } else if a < 0 {
                    let idx = ((TILE_SIZE as isize - 1 + isize::from(a)) * (TILE_SIZE as isize)
                        + isize::from(b)) as usize;
                    lnorth_tile[idx] as usize
                } else if b < 0 {
                    let idx = (isize::from(a) * (TILE_SIZE as isize)
                        + (TILE_SIZE as isize - 1 + isize::from(b)))
                        as usize;
                    lwest_tile[idx] as usize
                } else {
                    let idx = (isize::from(a) * (TILE_SIZE as isize) + isize::from(b)) as usize;
                    lmain_tile[idx] as usize
                };
                let power = states.pow(pw);
                ind += power * current_val;
                pw += 1;
            }
        }
        main_tile[0] = self.rule.get(ind);
        north_tile[(TILE_SIZE - 1) * TILE_SIZE] = self.rule.get(ind);
        west_tile[TILE_SIZE - 1] = self.rule.get(ind);
        northwest_tile[(TILE_SIZE - 1) * TILE_SIZE + TILE_SIZE - 1] = self.rule.get(ind);
    }

    #[inline]
    pub fn update(&mut self) {
        let bounds_high = self.n_tiles;
        //Main update
        for tx in 0..bounds_high {
            for ty in 0..bounds_high {
                self.update_tile(tx, ty);
            }
        }

        //Bounds update
        for tx in 0..bounds_high {
            for ty in 0..bounds_high {
                self.update_tile_boundaries(tx, ty);
            }
        }

        self.flop = !self.flop;
    }

    pub fn skipped_iter(&mut self, steps: u32, skip: u32) -> TiledAutomatonIterator {
        TiledAutomatonIterator {
            autom: self,
            skip,
            steps: Some(steps),
            ct: 0,
        }
    }
}

pub struct Automaton {
    pub size: usize,
    pub states: u8,
    flop: bool,
    grid1: Vec<u8>,
    grid2: Vec<u8>,
    rule: Rule,
}

impl Automaton {
    pub fn new(states: u8, size: usize, grid: Vec<u8>, rule: Rule) -> Automaton {
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

    pub fn skipped_iter(&mut self, steps: u32, skip: u32) -> AutomatonIterator {
        AutomatonIterator {
            autom: self,
            skip,
            steps: Some(steps),
            ct: 0,
        }
    }
}

pub struct AutomatonIterator<'a> {
    autom: &'a mut Automaton,
    skip: u32,
    steps: Option<u32>,
    ct: u32,
}

pub struct TiledAutomatonIterator<'a> {
    autom: &'a mut TiledAutomaton,
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

impl Iterator for TiledAutomatonIterator<'_> {
    type Item = TiledGrid;
    fn next(&mut self) -> Option<TiledGrid> {
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
    use super::test::Bencher;
    use crate::automaton::{Automaton, TiledAutomaton};
    use crate::rule::Rule;

    #[test]
    fn update_should_apply_rule() {
        let mut a = get_random_auto(32, 2);
        let b1 = a.flop;
        a.update();
        assert_ne!(b1, a.flop);
    }

    fn get_random_auto(size: usize, states: u8) -> Automaton {
        let states = super::test::black_box(states);
        let rule = Rule::random(1, states);
        let mut a = Automaton::new(states, size, vec![0; size * size], rule);
        a.random_init();
        a
    }

    fn get_random_tiled_auto(size: usize, states: u8) -> TiledAutomaton {
        let states = super::test::black_box(states);
        let rule = Rule::random(1, states);
        let mut a = TiledAutomaton::new(states, size, rule);
        a.random_init();
        a
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

    #[bench]
    fn bench_single_update_512_tiled(b: &mut Bencher) {
        let mut a = super::test::black_box(get_random_tiled_auto(512, 3));
        b.iter(|| a.update());
    }

    #[bench]
    fn bench_single_update_1024_tiled(b: &mut Bencher) {
        let mut a = super::test::black_box(get_random_tiled_auto(1024, 3));
        b.iter(|| a.update());
    }

    #[bench]
    fn bench_single_update_2048_tiled(b: &mut Bencher) {
        let mut a = super::test::black_box(get_random_tiled_auto(2048, 4));
        b.iter(|| a.update());
    }
}
