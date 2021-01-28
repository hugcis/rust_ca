mod automaton_base;
pub use automaton_base::Automaton;

mod tiled_automaton;
pub use tiled_automaton::{TiledAutomaton, TILE_SIZE};

type StepIteratorBox<'a> = Box<dyn Iterator<Item = Vec<u8>> + 'a>;

const HORIZON: i8 = 1;

pub trait AutomatonImpl {
    fn skipped_iter(&mut self, steps: u32, skip: u32, scale: u16) -> StepIteratorBox;
    fn get_size(&self) -> usize;
    fn get_states(&self) -> u8;
}
