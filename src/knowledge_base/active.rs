/// Active knowledge base trait
/// Extends the base knowledge base with the ability to actively query a target system
use crate::{knowledge_base::KnowledgeBaseTrait, word::Word};

/// Trait representing an active knowledge base that can submit queries to a target
pub trait ActiveKnowledgeBase: KnowledgeBaseTrait {
    /// Starts the target system for querying
    fn start_target(&mut self) -> Result<(), String>;

    /// Stops the target system after querying
    fn stop_target(&mut self) -> Result<(), String>;

    /// Submits a word to the target and returns the output
    fn submit_word(&mut self, word: &Word) -> Result<Word, String>;

    /// Gets the current state of the target (started or stopped)
    fn is_target_running(&self) -> bool;
}
