//! The cellular automata related utilities.
use std::error;
use std::fmt;
use std::fs::File;
use std::io::{self, BufRead};

mod automaton_base;
pub use automaton_base::Automaton;

mod tiled_automaton;
pub use tiled_automaton::{TiledAutomaton, TILE_SIZE};

type StepIteratorBox<'a> = Box<dyn Iterator<Item = Vec<u8>> + 'a>;

const HORIZON: i8 = 1;

/// The specifications for a starting pattern.
struct PatternSpec {
    /// The total number of states in the pattern.
    states: u8,
    /// The pattern background state (for inserting in a larger CA).
    background: u8,
    /// The pattern itself (2D grid).
    pattern: Vec<Vec<u8>>,
}

/// Error type for an error that happend during pattern parsing.
#[derive(Debug)]
pub enum PatternError {
    /// A io error during pattern parsing.
    PatternFileError(io::Error),
    /// A file format error during pattern parsing.
    PatternFormatError,
}

impl fmt::Display for PatternError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PatternError::PatternFormatError => write!(f, "incorrect pattern format in file"),
            // The wrapped error contains additional information and is available
            // via the source() method.
            PatternError::PatternFileError(..) => {
                write!(f, "io error with the pattern file")
            }
        }
    }
}

impl error::Error for PatternError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            PatternError::PatternFormatError => None,
            // The cause is the underlying implementation error type. Is implicitly
            // cast to the trait object `&error::Error`. This works because the
            // underlying type already implements the `Error` trait.
            PatternError::PatternFileError(ref e) => Some(e),
        }
    }
}

impl From<io::Error> for PatternError {
    fn from(err: io::Error) -> PatternError {
        PatternError::PatternFileError(err)
    }
}

/// Any cellular automaton implementation must implement this trait. This allows
/// users to use the CA without having to understand the underlying
/// implementation details.
pub trait AutomatonImpl {
    /// Makes a new cellular automaton with a given grid size, number of states
    /// per cells and rule.
    ///
    /// ```
    /// use rust_ca::rule::Rule;
    /// use rust_ca::automaton::Automaton;
    /// use rust_ca::automaton::AutomatonImpl;
    /// let automaton = Automaton::new(3, 128, Rule::random(1, 2));
    /// ```
    fn new(states: u8, size: usize, rule: crate::rule::Rule) -> Self;
    /// Creates a CA of a given size from a rule.
    /// ```
    /// # use rust_ca::rule::Rule;
    /// # use rust_ca::automaton::Automaton;
    /// # use rust_ca::automaton::AutomatonImpl;
    /// let automaton = Automaton::from_rule(Rule::random(1, 2), 128);
    /// ```
    fn from_rule(rule: crate::rule::Rule, size: usize) -> Self
    where
        Self: Sized,
    {
        Self::new(rule.states, size, rule)
    }
    /// Returns an boxed iterator of CA steps, skipping every `skip` step and
    /// scaling the grid by a factor `scale`. This is useful to output an
    /// animated CA with
    fn skipped_iter(&mut self, steps: u32, skip: u32, scale: u16) -> StepIteratorBox;
    /// Returns the size of the automaton.
    fn size(&self) -> usize;
    /// Returns the number of states of the automaton.
    fn states(&self) -> u8;
    /// Returns a boxed iterator of CA steps.
    fn iter(&mut self, steps: u32) -> StepIteratorBox {
        self.skipped_iter(steps, 0, 1)
    }
    /// Initializes all the cells of the grid from a pattern file.
    fn init_from_pattern(&mut self, pattern_fname: &str) -> Result<(), PatternError>;
    /// Performs a single step update of the CA grid according to the rule.
    fn update(&mut self);
    /// Randomly sets all the cells of the cellular automaton grid
    fn random_init(&mut self);
    /// Gets the current grid.
    fn grid(&self) -> Vec<u8>;
}

/// Parses a pattern file. This returns a PatternSpec or an error if the pattern
/// is incorrect.
fn parse_pattern(pattern_fname: &str) -> Result<PatternSpec, PatternError> {
    let mut background: u8 = 0;
    let mut states: u8 = 0;
    let mut begin_pattern = false;
    let mut pattern: Vec<Vec<u8>> = vec![];
    let pat_file = File::open(pattern_fname)?;
    for opt_line in io::BufReader::new(pat_file).lines() {
        let line = opt_line.map_err(|_| PatternError::PatternFormatError)?;
        if line.starts_with('#') {
            begin_pattern = !begin_pattern;
        } else if begin_pattern {
            pattern.push(line.chars().into_iter().map(|x| x as u8 - b'0').collect());
        } else if line.contains(&"=".to_string()) {
            let content: Vec<&str> = line.split('=').take(2).collect();
            match content[0] {
                "N" => {
                    states = content[1]
                        .parse()
                        .map_err(|_| PatternError::PatternFormatError)?
                }
                "BG" => {
                    background = content[1]
                        .parse()
                        .map_err(|_| PatternError::PatternFormatError)?;
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

/// This will copy the CA grid of size `size` and will duplicate cells with the
/// a `scale` factor for image generation.
#[inline]
fn duplicate_array(s: &[u8], size: usize, scale: u16) -> Vec<u8> {
    if scale > 1 {
        let scaled_size = size * scale as usize;
        let mut out = Vec::with_capacity(scaled_size * scaled_size);
        for i in 0..scaled_size {
            for j in 0..scaled_size {
                let item = s[(i / scale as usize) * size + (j / scale as usize)];
                out.push(item);
            }
        }
        out
    } else {
        Vec::from(s)
    }
}
