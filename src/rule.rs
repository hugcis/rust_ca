//! Functions and struct to create and manipulate CA rules.
extern crate rand_distr;
mod utils;

use std::convert::TryInto;
use std::fs::File;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::ops::{Index, IndexMut};
use std::path::Path;
use std::str::FromStr;

use flate2::read::GzDecoder;
use flate2::read::ZlibDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use rand::Rng;
use rand_distr::Dirichlet;
use rand_distr::Distribution;

const ALPHA: f64 = 0.2;
const GZIP_H: [u8; 9] = [0x1f, 0x8b, 0x08, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0];

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
            panic!(
                "Incorrect rule for neighborhood size {} and number of states {}",
                horizon, states
            )
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

    fn rule_size(horizon: i8, states: u8) -> u64 {
        (states as u64).pow((2 * horizon + 1).pow(2).try_into().unwrap())
    }

    /// Create a random rule with uniformly sampled transitions.
    pub fn random(horizon: i8, states: u8) -> Rule {
        let mut rng = rand::thread_rng();
        let big_bound: u64 = Rule::rule_size(horizon, states);
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
        let big_bound: u64 = Rule::rule_size(horizon, states);
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
    /// ```
    /// use rust_ca::rule::Rule;
    ///
    /// # let rule = Rule::random(1, 2);
    /// # rule.to_file("test_path.rule")?;
    /// let rule_from_file = Rule::from_file("test_path.rule")?;
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn from_file<P: AsRef<Path> + Copy>(path: P) -> Result<Rule, std::io::Error> {
        let mut f = File::open(path)?;
        let mut header_test = [0; 9];

        f.read_exact(&mut header_test)?;
        f.seek(SeekFrom::Start(0))?;

        let mut table = Vec::new();
        if !header_test.iter().zip(GZIP_H.iter()).all(|(a, b)| a == b) {
            let mut decoder = ZlibDecoder::new(f);
            decoder.read_to_end(&mut table)?;
        } else {
            let mut decoder = GzDecoder::new(f);
            decoder.read_to_end(&mut table)?;
        };
        let zero = '0';
        for i in &mut table {
            *i -= zero as u8;
        }

        // Infer the number of states and horizon from the table size
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
    /// The resulting file contains the zlib compressed bytes of the rule.
    ///
    ///
    ///
    /// ```
    /// use rust_ca::rule::Rule;
    ///
    /// let rule = Rule::random(1, 2);
    /// rule.to_file("test_path.rule")?;
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), std::io::Error> {
        let f = File::create(path)?;
        let mut encoder = GzEncoder::new(f, Compression::default());
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
        self.table.len() as u64 == Rule::rule_size(self.horizon, self.states)
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

    /// Symmetrize a rule
    pub fn symmetrize(&mut self) {
        let states = self.states;
        let side = (self.horizon * 2 + 1) as usize;
        let table = self.table_mut();
        let size = table.len();
        let mut book_keep = vec![false; size];

        for position in 0..size as u64 {
            if book_keep[position as usize] {
                continue;
            }
            let position_90 =
                reverse_rows_position(transpose_position(position, states, side), states, side)
                    as usize;
            let position_270 =
                reverse_cols_position(transpose_position(position, states, side), states, side)
                    as usize;
            let position_180 =
                reverse_cols_position(reverse_rows_position(position, states, side), states, side)
                    as usize;
            let position_reverse_r = reverse_rows_position(position, states, side) as usize;
            let position_reverse_c = reverse_cols_position(position, states, side) as usize;
            let position_tr = transpose_position(position, states, side) as usize;
            let position_atr = transpose_position(
                reverse_rows_position(transpose_position(position, states, side), states, side),
                states,
                side,
            ) as usize;

            let position = position as usize;
            book_keep[position] = true;
            book_keep[position_180] = true;
            book_keep[position_90] = true;
            book_keep[position_270] = true;
            book_keep[position_reverse_c] = true;
            book_keep[position_reverse_r] = true;
            book_keep[position_tr] = true;
            book_keep[position_atr] = true;

            table[position_180] = table[position];
            table[position_90] = table[position];
            table[position_270] = table[position];
            table[position_reverse_c] = table[position];
            table[position_reverse_r] = table[position];
            table[position_tr] = table[position];
            table[position_atr] = table[position];
        }
    }
}

fn transpose_position(position: u64, states: u8, side: usize) -> u64 {
    let mut new_pos = position;
    for i in 0..side {
        for j in i + 1..side {
            let pow = (states as u64).pow((i * side + j) as u32);
            let pow_tr = (states as u64).pow((j * side + i) as u32);
            let state_a = (position / pow) % (states as u64);
            let state_b = (position / pow_tr) % (states as u64);
            new_pos += state_a * pow_tr + state_b * pow;
            new_pos -= state_a * pow + state_b * pow_tr;
        }
    }
    new_pos
}

fn reverse_cols_position(position: u64, states: u8, side: usize) -> u64 {
    let mut new_pos = position;
    for i in 0..side {
        for j in 0..side {
            let pow = (states as u64).pow((i * side + j) as u32);
            let pow_inv = (states as u64).pow((i * side + side - j - 1) as u32);
            let state = (position / pow) % (states as u64);
            new_pos += state * pow_inv;
            new_pos -= state * pow;
        }
    }
    new_pos
}

fn reverse_rows_position(position: u64, states: u8, side: usize) -> u64 {
    let mut new_pos = position;
    for i in 0..side {
        for j in 0..side {
            let pow = (states as u64).pow((i * side + j) as u32);
            let pow_inv = (states as u64).pow(((side - i - 1) * side + j) as u32);
            let state = (position / pow) % (states as u64);
            new_pos += state * pow_inv;
            new_pos -= state * pow;
        }
    }
    new_pos
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
    use crate::rule::reverse_cols_position;
    use crate::rule::reverse_rows_position;

    use super::{transpose_position, Rule};

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

    #[test]
    fn encode_decode() -> Result<(), std::io::Error> {
        let rule = Rule::random(1, 3);
        let table_before = rule.table().clone();
        rule.to_file("test_encode_decode.rule")?;

        let rule_after = Rule::from_file("test_encode_decode.rule")?;
        assert!(rule_after
            .table()
            .iter()
            .zip(table_before.iter())
            .all(|(a, b)| a == b));
        Ok(())
    }

    // The numbers represent position of 2D neighborhoods CA and their transpose.
    #[test]
    fn should_transpose() {
        assert_eq!(transpose_position(86, 2, 3), 92);
        assert_eq!(transpose_position(342, 2, 3), 348);
        assert_eq!(transpose_position(70, 2, 3), 76);

        assert_eq!(transpose_position(31759728, 2, 5), 18698958);
        assert_eq!(transpose_position(14260627, 2, 5), 10049977);

        assert_eq!(transpose_position(663423371124, 3, 5), 573900715524);
    }

    #[test]
    fn should_reverse_cols() {
        assert_eq!(reverse_cols_position(4808293, 2, 5), 4519732);
        assert_eq!(reverse_cols_position(4562286, 2, 5), 5075790);

        assert_eq!(reverse_cols_position(400932635627, 3, 5), 205325034491);
    }

    #[test]
    fn should_reverse_rows() {
        assert_eq!(reverse_rows_position(236, 2, 3), 299);

        assert_eq!(reverse_rows_position(30692772, 2, 5), 4642077);

        assert_eq!(reverse_rows_position(642938107354, 3, 5), 621701730346);
    }

    #[test]
    fn gol_is_symmetric() {
        let mut gol = Rule::gol();
        let table_before = gol.table.clone();
        gol.symmetrize();

        assert!(gol
            .table()
            .iter()
            .zip(table_before.iter())
            .all(|(a, b)| a == b));
    }

    #[test]
    fn symmetrization_is_idempotent() {
        let mut rule = Rule::random(1, 2);
        rule.symmetrize();
        let table_before = rule.table.clone();
        rule.symmetrize();
        assert!(rule
            .table()
            .iter()
            .zip(table_before.iter())
            .all(|(a, b)| a == b));


        let mut rule = Rule::random(1, 3);
        rule.symmetrize();
        let table_before = rule.table.clone();
        rule.symmetrize();
        assert!(rule
            .table()
            .iter()
            .zip(table_before.iter())
            .all(|(a, b)| a == b));
    }
}
