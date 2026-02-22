use super::tree::KnowledgeTree;
use crate::query::OutputQuery;
use crate::word::Word;

use super::stats::KnowledgeBaseStats;

/// Trait for knowledge base implementations
pub trait KnowledgeBaseTrait {
    fn resolve_query(&mut self, query: &mut OutputQuery) -> Result<(), String>;
    fn add_word(&mut self, input_word: &Word, output_word: &Word) -> Result<(), String>;
}

/// Default knowledge base implementation
pub struct KnowledgeBase {
    knowledge_tree: KnowledgeTree,
    stats: KnowledgeBaseStats,
}

impl KnowledgeBase {
    pub fn new() -> Self {
        KnowledgeBase {
            knowledge_tree: KnowledgeTree::new(),
            stats: KnowledgeBaseStats::new(),
        }
    }

    pub fn resolve_query(&mut self, query: &mut OutputQuery) -> Result<(), String> {
        let output_word = self.resolve_word(&query.input_word)?;
        query.output_word = Some(output_word);
        Ok(())
    }

    fn resolve_word(&mut self, word: &Word) -> Result<Word, String> {
        self.stats.increment_nb_query();
        self.stats.add_nb_letter(word.len());

        match self.knowledge_tree.get_output_word(word) {
            Ok(output) => Ok(output),
            Err(_) => {
                self.stats.increment_nb_submitted_query();
                self.stats.add_nb_submitted_letter(word.len());

                let output = self.execute_word(word)?;
                self.knowledge_tree.add_word(word, &output)?;
                Ok(output)
            }
        }
    }

    fn execute_word(&self, _word: &Word) -> Result<Word, String> {
        Err("Passive inference process".to_string())
    }

    pub fn add_word(&mut self, input_word: &Word, output_word: &Word) -> Result<(), String> {
        self.knowledge_tree.add_word(input_word, output_word)
    }

    pub fn stats(&self) -> &KnowledgeBaseStats {
        &self.stats
    }

    pub fn stats_mut(&mut self) -> &mut KnowledgeBaseStats {
        &mut self.stats
    }
}

impl Default for KnowledgeBase {
    fn default() -> Self {
        Self::new()
    }
}

impl KnowledgeBaseTrait for KnowledgeBase {
    fn resolve_query(&mut self, query: &mut OutputQuery) -> Result<(), String> {
        self.resolve_query(query)
    }

    fn add_word(&mut self, input_word: &Word, output_word: &Word) -> Result<(), String> {
        self.add_word(input_word, output_word)
    }
}
