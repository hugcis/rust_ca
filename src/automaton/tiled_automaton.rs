use super::{parse_pattern, AutomatonImpl, PatternError, HORIZON};
use crate::automaton::duplicate_array;
use crate::rule::Rule;
use rand::Rng;

/// The size of tiles in the tiled cellular automaton.
pub const TILE_SIZE: usize = 257;

pub type TiledGrid = Vec<[u8; TILE_SIZE * TILE_SIZE]>;

/// A tiled version of the cellular automaton for more cache-friendly simulation
/// on large grids.
pub struct TiledAutomaton {
    size: usize,
    n_tiles: usize,
    states: u8,
    flop: bool,
    grid1: TiledGrid,
    grid2: TiledGrid,
    rule: Rule,
}

impl TiledAutomaton {
    #[inline]
    fn grid_mut(&mut self) -> &mut TiledGrid {
        if self.flop {
            &mut self.grid1
        } else {
            &mut self.grid2
        }
    }

    #[inline]
    fn prev_grid(&mut self) -> &mut TiledGrid {
        if self.flop {
            &mut self.grid2
        } else {
            &mut self.grid1
        }
    }

    #[inline]
    fn update_tile(&mut self, tx: usize, ty: usize) {
        let n_tiles = self.n_tiles;
        let states = self.states as usize;
        let grid = self.grid_mut()[tx * n_tiles + ty];
        for i in HORIZON as usize..TILE_SIZE - HORIZON as usize {
            for j in HORIZON as usize..TILE_SIZE - HORIZON as usize {
                let is = i as isize;
                let js = j as isize;
                let mut ind: usize = 0;
                let mut pw = 0;
                for a in -HORIZON..=HORIZON {
                    for b in -HORIZON..=HORIZON {
                        let idx =
                            ((is + a as isize) * (TILE_SIZE as isize) + (js + b as isize)) as usize;
                        let current_val = grid[idx] as usize;
                        let power = states.pow(pw);
                        ind += power * current_val;
                        pw += 1;
                    }
                }
                self.prev_grid()[tx * n_tiles + ty][i * TILE_SIZE + j] = self.rule[ind];
            }
        }
    }

    #[inline]
    fn update_tile_boundaries(&mut self, tx: usize, ty: usize) {
        let states = self.states as usize;
        let n_tiles = self.n_tiles;
        let prev_x = (tx + self.n_tiles - 1) % self.n_tiles;
        let prev_y = (ty + self.n_tiles - 1) % self.n_tiles;
        let lmain_tile = self.grid_mut()[tx * n_tiles + ty];
        let lnorth_tile = self.grid_mut()[prev_x * n_tiles + ty];
        let lwest_tile = self.grid_mut()[tx * n_tiles + prev_y];
        let lnorthwest_tile = self.grid_mut()[prev_x * n_tiles + prev_y];

        for i in 1..TILE_SIZE - 1 {
            let is = i as isize;
            let mut ind: usize = 0;
            let mut pw = 0;
            for a in -HORIZON..=HORIZON {
                for b in -HORIZON..=HORIZON {
                    let current_val = if b < 0 {
                        let idx = ((is + a as isize) * (TILE_SIZE as isize)
                            + (TILE_SIZE as isize - 1 + b as isize))
                            as usize;
                        lwest_tile[idx] as usize
                    } else {
                        let idx = ((is + a as isize) * (TILE_SIZE as isize) + b as isize) as usize;
                        lmain_tile[idx] as usize
                    };
                    let power = states.pow(pw);
                    ind += power * current_val;
                    pw += 1;
                }
            }
            self.prev_grid()[tx * n_tiles + ty][i * TILE_SIZE] = self.rule[ind];
            self.prev_grid()[tx * n_tiles + prev_y][i * TILE_SIZE + (TILE_SIZE - 1)] =
                self.rule[ind];
        }
        for j in 1..TILE_SIZE - 1 {
            let js = j as isize;
            let mut ind: usize = 0;
            let mut pw = 0;
            for a in -HORIZON..=HORIZON {
                for b in -HORIZON..=HORIZON {
                    let current_val = if a < 0 {
                        let idx = ((TILE_SIZE as isize - 1 + a as isize) * (TILE_SIZE as isize)
                            + (js + b as isize)) as usize;
                        lnorth_tile[idx] as usize
                    } else {
                        let idx = (a as isize * (TILE_SIZE as isize) + js + b as isize) as usize;
                        lmain_tile[idx] as usize
                    };
                    let power = states.pow(pw);
                    ind += power * current_val;
                    pw += 1;
                }
            }
            self.prev_grid()[tx * n_tiles + ty][j] = self.rule[ind];
            self.prev_grid()[prev_x * n_tiles + ty][(TILE_SIZE - 1) * TILE_SIZE + j] =
                self.rule[ind];
        }

        let mut ind: usize = 0;
        let mut pw = 0;
        for a in -HORIZON..=HORIZON {
            for b in -HORIZON..=HORIZON {
                let current_val = if (a < 0) & (b < 0) {
                    let idx = ((TILE_SIZE as isize - 1 + a as isize) * (TILE_SIZE as isize)
                        + (TILE_SIZE as isize - 1 + b as isize))
                        as usize;
                    lnorthwest_tile[idx] as usize
                } else if a < 0 {
                    let idx = ((TILE_SIZE as isize - 1 + a as isize) * (TILE_SIZE as isize)
                        + b as isize) as usize;
                    lnorth_tile[idx] as usize
                } else if b < 0 {
                    let idx = (a as isize * (TILE_SIZE as isize)
                        + (TILE_SIZE as isize - 1 + b as isize))
                        as usize;
                    lwest_tile[idx] as usize
                } else {
                    let idx = (a as isize * (TILE_SIZE as isize) + b as isize) as usize;
                    lmain_tile[idx] as usize
                };
                let power = states.pow(pw);
                ind += power * current_val;
                pw += 1;
            }
        }
        self.prev_grid()[tx * n_tiles + ty][0] = self.rule[ind];
        self.prev_grid()[prev_x * n_tiles + ty][(TILE_SIZE - 1) * TILE_SIZE] = self.rule[ind];
        self.prev_grid()[tx * n_tiles + prev_y][TILE_SIZE - 1] = self.rule[ind];
        self.prev_grid()[prev_x * n_tiles + prev_y][(TILE_SIZE - 1) * TILE_SIZE + TILE_SIZE - 1] =
            self.rule[ind];
    }
}

impl AutomatonImpl for TiledAutomaton {
    fn new(states: u8, size: usize, rule: Rule) -> TiledAutomaton {
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

    #[inline]
    fn grid(&self) -> Vec<u8> {
        duplicate_array_tiled(
            if self.flop { &self.grid1 } else { &self.grid2 },
            self.size,
            1,
        )
    }

    fn skipped_iter(
        &mut self,
        steps: u32,
        skip: u32,
        scale: u16,
    ) -> Box<dyn Iterator<Item = Vec<u8>> + '_> {
        let size = self.size;
        Box::new(
            TiledAutomatonIterator {
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

    fn init_from_pattern(&mut self, pattern_fname: &str) -> Result<(), PatternError> {
        let pattern_spec = parse_pattern(pattern_fname)?;
        assert!(pattern_spec.states <= self.states);
        assert!(pattern_spec.background < self.states);
        for i in self.grid_mut().iter_mut() {
            for j in i.iter_mut() {
                *j = pattern_spec.background;
            }
        }
        let lines = pattern_spec.pattern.len();
        let cols = pattern_spec.pattern.iter().map(|x| x.len()).max().unwrap();
        let n_tiles = self.n_tiles;
        for i in 0..lines {
            let lin = &pattern_spec.pattern[i];
            for (j, elem) in lin.iter().enumerate() {
                let idx_x = i + (self.size / 2) - lines / 2;
                let idx_y = j - cols / 2 + self.size / 2;
                let tx = idx_x / TILE_SIZE;
                let ty = idx_y / TILE_SIZE;
                let x = idx_x % TILE_SIZE;
                let y = idx_y % TILE_SIZE;
                self.grid_mut()[tx * n_tiles + ty][x * TILE_SIZE + y] = *elem;
            }
        }
        Ok(())
    }

    #[inline]
    fn update(&mut self) {
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
        // Flip buffer choice
        self.flop = !self.flop;
    }

    fn random_init(&mut self) {
        let states = self.states;
        let mut rng = rand::thread_rng();
        for i in self.grid_mut().iter_mut() {
            for j in i.iter_mut() {
                *j = rng.gen_range(0..states);
            }
        }
    }
}

pub struct TiledAutomatonIterator<'a> {
    autom: &'a mut TiledAutomaton,
    skip: u32,
    steps: Option<u32>,
    ct: u32,
}

impl Iterator for TiledAutomatonIterator<'_> {
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
fn duplicate_array_tiled(s: &[[u8; TILE_SIZE * TILE_SIZE]], size: usize, scale: u16) -> Vec<u8> {
    let scaled_size = size * scale as usize;
    let n_tiles = size / TILE_SIZE;
    let mut out = Vec::with_capacity(scaled_size * scaled_size);
    for a in 0..scaled_size {
        for b in 0..scaled_size {
            let idx_i = a / scale as usize;
            let idx_j = b / scale as usize;
            let tx = idx_i / (TILE_SIZE - 1);
            let ty = idx_j / (TILE_SIZE - 1);
            let idx_x = idx_i % (TILE_SIZE - 1);
            let idx_y = idx_j % (TILE_SIZE - 1);
            let item = s[tx * n_tiles + ty][idx_x * TILE_SIZE + idx_y];
            out.push(item);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use crate::automaton::AutomatonImpl;
    use crate::automaton::TiledAutomaton;
    use crate::rule::Rule;
    use test::Bencher;

    fn get_random_tiled_auto(size: usize, states: u8) -> TiledAutomaton {
        let states = test::black_box(states);
        let rule = Rule::random(1, states);
        let mut a = TiledAutomaton::new(states, size, rule);
        a.random_init();
        a
    }

    #[test]
    fn update_should_apply_rule() {
        let mut a = get_random_tiled_auto(32, 2);
        let b1 = a.flop;
        a.update();
        assert_ne!(b1, a.flop);
    }

    #[bench]
    fn bench_single_update_512_tiled(b: &mut Bencher) {
        let mut a = test::black_box(get_random_tiled_auto(512, 3));
        b.iter(|| a.update());
    }

    #[bench]
    fn bench_single_update_1024_tiled(b: &mut Bencher) {
        let mut a = test::black_box(get_random_tiled_auto(1024, 3));
        b.iter(|| a.update());
    }

    #[bench]
    fn bench_single_update_2048_tiled(b: &mut Bencher) {
        let mut a = test::black_box(get_random_tiled_auto(2048, 4));
        b.iter(|| a.update());
    }
}
