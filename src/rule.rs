use std::convert::TryInto;
use std::fs::File;
use std::io::Read;

use rand::Rng;
use rand_distr::Dirichlet;
use rand_distr::Distribution;
use flate2::read::ZlibDecoder;

const ALPHA: f64 = 0.4;

pub struct Rule {
    pub horizon: i8,
    states: u8,
    table: Vec<u8>,
}

impl Rule {
    pub fn new(horizon: i8, states: u8, table: Vec<u8>) -> Rule {
        let r = Rule {
            horizon,
            states,
            table,
        };
        if r.check() {
            r
        } else {
            panic!("Incorrect rule given")
        }
    }
    pub fn get(&self, index: usize) -> u8 {
        self.table[index]
    }

    pub fn random(horizon: i8, states: u8) -> Rule {
        let mut rng = rand::thread_rng();
        let big_bound: u64 = (states as u64).pow((2 * horizon + 1).pow(2).try_into().unwrap());
        let table: Vec<u8> = (0..big_bound).map(|_| rng.gen_range(0..states)).collect();
        Rule {
            horizon,
            states,
            table,
        }
    }

    pub fn random_dirichlet(horizon: i8, states: u8, alpha: Option<f64>) -> Rule {
        let alpha = match alpha {
            Some(v) => v,
            None => ALPHA,
        };
        let dirichlet = Dirichlet::new_with_size(alpha, states.into()).unwrap();
        let lambdas: Vec<f64> = dirichlet
            .sample(&mut rand::thread_rng())
            .iter()
            .scan(0., |acc, &x| {
                *acc += x;
                Some(*acc)
            })
            .collect();
        let big_bound: u64 = (states as u64).pow((2 * horizon + 1).pow(2).try_into().unwrap());
        let table: Vec<u8> = (0..big_bound)
            .map(|_| get_rand_state(&lambdas, states))
            .collect();
        Rule {
            horizon,
            states,
            table,
        }
    }

    /// Read a rule from specified filename.
    pub fn from_file(fname: &str) -> Result<Rule, std::io::Error> {
        let f = File::open(fname)?;
        let mut decoder = ZlibDecoder::new(f);
        let mut table = Vec::new();
        decoder.read_to_end(&mut table)?;
        let zero = '0';
        for i in &mut table {
            *i -= zero as u8;
        }
        let (states, horizon) = (2..30)
            .find_map(|i| {
                let d = (table.len() as f64).ln() / (i as f64).ln();
                if (d - d.floor()).abs() < f64::EPSILON
                    && (d.sqrt() - d.sqrt().floor()).abs() < f64::EPSILON
                {
                    Some((i, ((d.sqrt() - 1.) / 2.) as i8))
                } else {
                    None
                }
            })
            .unwrap();
        Ok(Rule::new(horizon, states, table))
    }

    pub fn check(&self) -> bool {
        self.table.len() as u64
            == (self.states as u64).pow((2 * self.horizon + 1).pow(2).try_into().unwrap())
    }
}

fn get_rand_state(lambdas: &[f64], states: u8) -> u8 {
    assert_eq!(lambdas.len(), states.into());
    let mut rng = rand::thread_rng();
    let val: f64 = rng.gen_range(0.0..1.0);
    lambdas
        .iter()
        .enumerate()
        .rev()
        .find_map(|(idx, l)| if val >= *l { Some(idx as u8 + 1) } else { None })
        .unwrap_or(0)
}
