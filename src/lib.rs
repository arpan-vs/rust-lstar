//! Rust implementation of the L* active automata learning algorithm.
//!
//! This crate provides:
//! - Core automata data structures (`Automata`, `State`, `Transition`)
//! - Knowledge base abstractions for answering membership queries
//! - Multiple equivalence-test strategies
//! - An `LSTAR` learner that infers Mealy-style automata from observations

/// Automata model and DOT import/export helpers.
pub mod automata;
/// Equivalence-test strategies used by the learner.
pub mod eqtest;
/// Knowledge-base traits and implementations for query resolution.
pub mod knowledge_base;
/// Letter/symbol representation.
pub mod letter;
/// Main L* learner implementation.
pub mod lstar;
/// Observation-table implementation used by L*.
pub mod observation_table;
/// Query data model.
pub mod query;
/// Word abstraction built from letters.
pub mod word;
// `automata` now contains `state`, `transition`, and `dot_parser` submodules

/// Deterministic transducer representation.
pub use automata::Automata;
/// Automaton state.
pub use automata::State;
/// Automaton transition.
pub use automata::Transition;
/// Parse a DOT graph into an [`Automata`].
pub use automata::{build_dot_code, parse_dot};
/// BDist equivalence testing strategy.
pub use eqtest::BDistMethod;
/// Trait implemented by equivalence testing strategies.
pub use eqtest::EquivalenceTest;
/// Composition of multiple equivalence testing strategies.
pub use eqtest::MultipleEqtests;
/// Random-walk equivalence testing strategy.
pub use eqtest::RandomWalkMethod;
/// W-method equivalence testing strategy.
pub use eqtest::WMethodEQ;
/// Alias for [`WMethodEQ`].
pub use eqtest::WpMethodEQ;
/// Default caching knowledge base.
pub use knowledge_base::KnowledgeBase;
/// Knowledge tree data structures used by knowledge bases.
pub use knowledge_base::{KnowledgeNode, KnowledgeTree};
/// Letter model and empty-letter marker.
pub use letter::{EmptyLetter, Letter};
/// L* learner entry point.
pub use lstar::LSTAR;
/// Observation table used by the learner.
pub use observation_table::ObservationTable;
/// Membership query structure.
pub use query::OutputQuery;
/// Word model.
pub use word::Word;
