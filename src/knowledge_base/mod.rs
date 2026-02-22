//! Knowledge-base traits and implementations.
//!
//! Knowledge bases answer membership queries issued by the learner and can
//! optionally interact with live systems.

/// Active querying abstraction.
pub mod active;
/// Default cached knowledge-base implementation.
pub mod base;
/// In-memory fake active knowledge base backed by an automaton.
pub mod fake;
/// TCP-based active knowledge base.
pub mod network;
/// Query/letter counters and reporting helpers.
pub mod stats;
/// Tree cache used to store input/output mappings.
pub mod tree;

/// Trait for active knowledge bases.
pub use active::ActiveKnowledgeBase;
/// Base knowledge-base trait and default implementation.
pub use base::{KnowledgeBase, KnowledgeBaseTrait};
/// Fake active knowledge base.
pub use fake::FakeActiveKnowledgeBase;
/// Network-backed active knowledge base.
pub use network::NetworkActiveKnowledgeBase;
/// Knowledge-base statistics container.
pub use stats::KnowledgeBaseStats;
/// Knowledge tree structures.
pub use tree::{KnowledgeNode, KnowledgeTree};
