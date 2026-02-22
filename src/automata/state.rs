use super::transition::Transition;
use crate::letter::Letter;
use std::fmt;

/// A state in an automaton.
#[derive(Clone, Debug)]
pub struct State {
    /// Human-readable unique state name.
    pub name: String,
    /// Outgoing transitions from this state.
    pub transitions: Vec<Transition>,
}

impl State {
    /// Create a new state with no outgoing transitions.
    pub fn new(name: String) -> Self {
        State {
            name,
            transitions: Vec::new(),
        }
    }

    /// Add an outgoing transition.
    pub fn add_transition(&mut self, transition: Transition) {
        self.transitions.push(transition);
    }

    /// Get a transition for a given input letter
    pub fn get_transition(&self, input_letter: &Letter) -> Option<&Transition> {
        self.transitions
            .iter()
            .find(|t| t.input_letter == *input_letter)
    }

    /// Follow a transition for the given input letter.
    ///
    /// Returns the emitted output letter and destination state when a matching
    /// transition exists.
    pub fn visit(&self, input_letter: &Letter) -> Option<(&Letter, &State)> {
        for transition in &self.transitions {
            if &transition.input_letter == input_letter {
                return Some((&transition.output_letter, &transition.output_state));
            }
        }
        None
    }
}

impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for State {}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}
