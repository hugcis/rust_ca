use std::fs::File;
use std::io::{self, BufRead};

mod automaton_base;
pub use automaton_base::Automaton;

mod tiled_automaton;
pub use tiled_automaton::{TiledAutomaton, TILE_SIZE};

type StepIteratorBox<'a> = Box<dyn Iterator<Item = Vec<u8>> + 'a>;

const HORIZON: i8 = 1;

pub struct PatternSpec {
    states: u8,
    background: u8,
    pattern: Vec<Vec<u8>>,
}

pub trait AutomatonImpl {
    fn skipped_iter(&mut self, steps: u32, skip: u32, scale: u16) -> StepIteratorBox;
    fn get_size(&self) -> usize;
    fn get_states(&self) -> u8;
    fn parse_pattern(&self, pattern_fname: &str) -> Result<PatternSpec, io::Error> {
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
}
