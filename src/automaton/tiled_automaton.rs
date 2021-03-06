use super::{AutomatonImpl, HORIZON};
use crate::rule::Rule;
use rand::Rng;

pub const TILE_SIZE: usize = 129;

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
                self.prev_grid()[tx * n_tiles + ty][i * TILE_SIZE + j] = self.rule.get(ind);
            }
        }
    }

    #[inline]
    pub fn update_tile_boundaries(&mut self, tx: usize, ty: usize) {
        let states = self.states as usize;
        let n_tiles = self.n_tiles;
        let prev_x = (tx + self.n_tiles - 1) % self.n_tiles;
        let prev_y = (ty + self.n_tiles - 1) % self.n_tiles;
        let lmain_tile = self.grid()[tx * n_tiles + ty];
        let lnorth_tile = self.grid()[prev_x * n_tiles + ty];
        let lwest_tile = self.grid()[tx * n_tiles + prev_y];
        let lnorthwest_tile = self.grid()[prev_x * n_tiles + prev_y];

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
            self.prev_grid()[tx * n_tiles + ty][i * TILE_SIZE] = self.rule.get(ind);
            self.prev_grid()[tx * n_tiles + prev_y][i * TILE_SIZE + (TILE_SIZE - 1)] =
                self.rule.get(ind);
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
            self.prev_grid()[tx * n_tiles + ty][j] = self.rule.get(ind);
            self.prev_grid()[prev_x * n_tiles + ty][(TILE_SIZE - 1) * TILE_SIZE + j] =
                self.rule.get(ind);
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
        self.prev_grid()[tx * n_tiles + ty][0] = self.rule.get(ind);
        self.prev_grid()[prev_x * n_tiles + ty][(TILE_SIZE - 1) * TILE_SIZE] = self.rule.get(ind);
        self.prev_grid()[tx * n_tiles + prev_y][TILE_SIZE - 1] = self.rule.get(ind);
        self.prev_grid()[prev_x * n_tiles + prev_y][(TILE_SIZE - 1) * TILE_SIZE + TILE_SIZE - 1] =
            self.rule.get(ind);
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
        // Flip buffer choice
        self.flop = !self.flop;
    }
}

impl AutomatonImpl for TiledAutomaton {
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
            .map(move |grid| duplicate_array_tiled(grid, size, scale)),
        )
    }

    fn get_size(&self) -> usize {
        self.size
    }

    fn get_states(&self) -> u8 {
        self.states
    }
}

pub struct TiledAutomatonIterator<'a> {
    autom: &'a mut TiledAutomaton,
    skip: u32,
    steps: Option<u32>,
    ct: u32,
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

#[inline]
fn duplicate_array_tiled(s: TiledGrid, size: usize, scale: u16) -> Vec<u8> {
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
    use crate::rule::Rule;
    use crate::TiledAutomaton;
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
