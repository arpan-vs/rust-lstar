//! Automata primitives and DOT serialization/parsing utilities.

/// Core automaton type and execution helpers.
pub mod automata;
/// Lightweight parser/serializer for DOT representation.
pub mod dot_parser;
/// State type.
pub mod state;
/// Transition type.
pub mod transition;

/// Complete automaton model.
pub use self::automata::Automata;
/// DOT parser and serializer helpers.
pub use self::dot_parser::{build_dot_code, parse_dot};
/// State model.
pub use self::state::State;
/// Transition model.
pub use self::transition::Transition;
