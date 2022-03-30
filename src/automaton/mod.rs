//! The cellular automata related utilities.
use std::fs::File;
use std::io::{self, BufRead};

mod automaton_base;
pub use automaton_base::Automaton;

mod tiled_automaton;
pub use tiled_automaton::{TiledAutomaton, TILE_SIZE};

type StepIteratorBox<'a> = Box<dyn Iterator<Item = Vec<u8>> + 'a>;

const HORIZON: i8 = 1;

struct PatternSpec {
    states: u8,
    background: u8,
    pattern: Vec<Vec<u8>>,
}

/// An automaton must implement this trait.
pub trait AutomatonImpl {
    /// Make a new cellular automaton with a given grid size, number of states per cells and rule.
    ///
    /// ```
    /// use rust_ca::rule::Rule;
    /// use rust_ca::automaton::Automaton;
    /// use rust_ca::automaton::AutomatonImpl;
    /// let automaton = Automaton::new(3, 128, Rule::random(1, 2));
    /// ```
    fn new(states: u8, size: usize, rule: crate::rule::Rule) -> Self;
    /// Returns an boxed iterator of CA steps, skipping every `skip` step and
    /// scaling the grid by a factor `scale`.
    fn skipped_iter(&mut self, steps: u32, skip: u32, scale: u16) -> StepIteratorBox;
    /// Returns the size of the automaton.
    fn size(&self) -> usize;
    /// Returns the number of states of the automaton.
    fn states(&self) -> u8;
    /// Returns a boxed iterator of CA steps.
    fn iter(&mut self, steps: u32) -> StepIteratorBox {
        self.skipped_iter(steps, 0, 1)
    }
    /// Initialize all the cells of the grid from a pattern file.
    fn init_from_pattern(&mut self, pattern_fname: &str);
    /// Perform a single step update of the CA grid according to the rule.
    fn update(&mut self);
    /// Randomly set all the cells of the cellular automaton grid
    fn random_init(&mut self);
    /// Get the current grid.
    fn grid(&self) -> Vec<u8>;
}

fn parse_pattern(pattern_fname: &str) -> Result<PatternSpec, io::Error> {
    let mut background: u8 = 0;
    let mut states: u8 = 0;
    let mut begin_pattern = false;
    let mut pattern: Vec<Vec<u8>> = vec![];
    let pat_file = File::open(pattern_fname)?;
    for opt_line in io::BufReader::new(pat_file).lines() {
        let line = opt_line.unwrap();
        if line.starts_with('#') {
            begin_pattern = !begin_pattern;
        } else if begin_pattern {
            pattern.push(line.chars().into_iter().map(|x| x as u8 - b'0').collect());
        } else if line.contains(&"=".to_string()) {
            let content: Vec<&str> = line.split('=').take(2).collect();
            match content[0] {
                "N" => states = content[1].parse().unwrap(),
                "BG" => {
                    background = content[1].parse().unwrap_or_else(|_| {
                        panic!("BG= should be followed by int, found {}", content[1])
                    });
                }
                _ => {}
            }
        }
    }
    Ok(PatternSpec {
        states,
        background,
        pattern,
    })
}

#[inline]
fn duplicate_array(s: &[u8], size: usize, scale: u16) -> Vec<u8> {
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
