//! Equivalence-test strategy interfaces and implementations.
//!
//! These strategies search for a counterexample that disproves a current
//! hypothesis automaton.

use crate::word::Word;

/// BDist method implementation.
pub mod bdist_method;
/// Composition of multiple equivalence tests.
pub mod multiple_eqtests;
/// Random-walk based equivalence testing.
pub mod random_walk;
/// W-method implementation.
pub mod w_method;

/// BDist equivalence-test strategy.
pub use bdist_method::BDistMethod;
/// Composite equivalence-test strategy.
pub use multiple_eqtests::MultipleEqtests;
/// Random-walk equivalence-test strategy.
pub use random_walk::RandomWalkMethod;
/// W-method equivalence-test strategy.
pub use w_method::WMethodEQ;
/// Alias for the Wp-method name used by some literature.
pub type WpMethodEQ = WMethodEQ;

/// Represents a counterexample found during equivalence testing
#[derive(Clone, Debug)]
pub struct Counterexample {
    /// Input sequence that exposes the mismatch.
    pub input_word: Word,
    /// Observed output sequence from the system under learning.
    pub output_word: Word,
}

/// Trait for equivalence testing methods
pub trait EquivalenceTest {
    /// Find a counterexample for the given hypothesis
    /// Returns None if the automaton is equivalent to the system under learning
    fn find_counterexample(&self, hypothesis: &mut crate::automata::Automata)
        -> Option<Counterexample>;
}
