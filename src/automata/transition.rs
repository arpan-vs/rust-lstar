use crate::{letter::Letter, State};
use std::fmt;

/// A labeled transition between two states.
#[derive(Clone, Debug)]
pub struct Transition {
    /// Transition identifier.
    pub name: String,
    /// Source-state name used by flat transition representations.
    pub source_state: State,
    /// Destination state.
    pub output_state: State,
    /// Input letter consumed by the transition.
    pub input_letter: Letter,
    /// Output letter produced by the transition.
    pub output_letter: Letter,
}

impl Transition {
    /// Create a transition without explicit source-state metadata.
    pub fn new(
        name: String,
        source_state: State,
        output_state: State,
        input_letter: Letter,
        output_letter: Letter,
    ) -> Self {
        Transition {
            name,
            source_state,
            output_state,
            input_letter,
            output_letter,
        }
    }

    /// Build the `"input / output"` label used in visualizations.
    pub fn label(&self) -> String {
        format!(
            "{} / {}",
            self.input_letter.symbols(),
            self.output_letter.symbols()
        )
    }
}

impl fmt::Display for Transition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} / {}", self.input_letter, self.output_letter)
    }
}
