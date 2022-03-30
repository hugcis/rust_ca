//! Functions and struct to create and manipulate CA rules.
extern crate rand_distr;
mod utils;

use std::convert::TryInto;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::ops::{Index, IndexMut};
use std::path::Path;
use std::str::FromStr;

use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use rand::Rng;
use rand_distr::Dirichlet;
use rand_distr::Distribution;

const ALPHA: f64 = 0.2;

#[derive(Debug)]
/// The sampling mode for the random rule generation.
pub enum SamplingMode {
    /// Uniformly sample transitions in the rule table.
    Uniform,
    /// Sample transitions in the rule table according to a Dirichlet distribution.
    Dirichlet,
}

// Implement the FromStr trait for CLI options parsing.
impl FromStr for SamplingMode {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "uniform" => Ok(SamplingMode::Uniform),
            "dirichlet" => Ok(SamplingMode::Dirichlet),
            _ => Err("no match"),
        }
    }
}

#[derive(Debug, Clone)]
/// The rule object. Represents a cellular automaton rule.
pub struct Rule {
    horizon: i8,
    states: u8,
    table: Vec<u8>,
}

impl Rule {
    /// Construct a new rule from a given number of states, horizon and rule
    /// table
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

    /// Returns a reference to the rule table
    pub fn table(&self) -> &[u8] {
        &self.table
    }

    /// Returns a mutable reference to the rule table
    pub fn table_mut(&mut self) -> &mut Vec<u8> {
        &mut self.table
    }

    /// Create a random rule with uniformly sampled transitions.
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

    /// Create a random rule with transitions sampled according to a Dirichlet
    /// distribution with parameter `alpha`.
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
            .map(|_| rand_state(&lambdas, states))
            .collect();
        Rule {
            horizon,
            states,
            table,
        }
    }

    /// Read a rule from specified filename.
    /// ```ignore
    /// use rust_ca::rule::Rule;
    ///
    /// let rule_from = Rule::from_file("test_path.rule")?;
    /// ```
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Rule, std::io::Error> {
        let f = File::open(path)?;
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

    /// Write a compressed representation of the rule to a specified filename.
    /// ```
    /// use rust_ca::rule::Rule;
    ///
    /// let rule = Rule::random(1, 2);
    /// rule.to_file("test_path.rule")?;
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), std::io::Error> {
        let f = File::create(path)?;
        let mut encoder = ZlibEncoder::new(f, Compression::default());
        let zero = '0';
        let mut out_vec = Vec::new();
        for i in &self.table {
            out_vec.push(i + zero as u8);
        }
        encoder.write_all(&out_vec)?;
        encoder.try_finish()
    }

    /// Perform some checks on the rule to ensure its correctness.
    /// ```
    /// use rust_ca::rule::Rule;
    ///
    /// let mut rule = Rule::random(1, 2);
    /// assert!(rule.check());
    ///
    /// rule.table_mut().push(0);
    /// assert!(!rule.check());
    /// ```
    pub fn check(&self) -> bool {
        self.table.len() as u64
            == (self.states as u64).pow((2 * self.horizon + 1).pow(2).try_into().unwrap())
    }

    /// Returns the game of life rule.
    /// ```
    /// use rust_ca::rule::Rule;
    ///
    /// let gol_rule = Rule::gol();
    /// // Next central cell state for position:
    /// // 1 1 1
    /// // 0 1 0
    /// // 0 0 0
    /// // A live cell with 3 live neighbors lives on.
    /// assert_eq!(gol_rule[1 * 2_usize.pow(0) + 1 * 2_usize.pow(1) + 1 * 2_usize.pow(2) +
    ///                     1 * 2_usize.pow(4)], 1);
    /// // 1 1 1
    /// // 0 1 1
    /// // 0 0 0
    /// // A live cell with 4 live neighbors dies.
    /// assert_eq!(gol_rule[1 * 2_usize.pow(0) + 1 * 2_usize.pow(1) + 1 * 2_usize.pow(2) +
    ///                     1 * 2_usize.pow(4) + 1 * 2_usize.pow(5)], 0);
    /// ```
    pub fn gol() -> Self {
        Rule::new(1, 2, utils::GOL.to_vec())
    }
}

impl Index<usize> for Rule {
    type Output = u8;
    fn index(&self, idx: usize) -> &Self::Output {
        &self.table[idx]
    }
}

impl IndexMut<usize> for Rule {
    fn index_mut(&mut self, idx: usize) -> &mut Self::Output {
        &mut self.table[idx]
    }
}

fn rand_state(lambdas: &[f64], states: u8) -> u8 {
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

#[cfg(test)]
mod tests {
    use super::Rule;

    #[test]
    fn should_check_correct_rule_size() {
        let mut rule = Rule {
            states: 2,
            horizon: 1,
            table: vec![1; 512],
        };
        assert!(rule.check());
        rule.table.push(0);
        assert!(!rule.check());

        rule = Rule {
            states: 3,
            horizon: 1,
            table: vec![1; 19683],
        };
        assert!(rule.check());
        rule.table.push(0);
        assert!(!rule.check());
    }
}
