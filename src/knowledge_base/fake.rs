use super::active::ActiveKnowledgeBase;
use super::base::{KnowledgeBase, KnowledgeBaseTrait};
use crate::automata::Automata;
use crate::letter::Letter;
use crate::query::OutputQuery;
/// Fake active knowledge base implementation
/// Uses a preset automata to simulate target behavior
use crate::word::Word;

/// A test implementation of an active knowledge base that uses a preset automata
pub struct FakeActiveKnowledgeBase {
    base: KnowledgeBase,
    automata: Option<Automata>,
    target_running: bool,
}

impl FakeActiveKnowledgeBase {
    /// Creates a new fake active knowledge base with the given automata
    pub fn new(automata: Automata) -> Self {
        FakeActiveKnowledgeBase {
            base: KnowledgeBase::new(),
            automata: Some(automata),
            target_running: false,
        }
    }

    /// Gets a reference to the automata
    pub fn automata(&self) -> Option<&Automata> {
        self.automata.as_ref()
    }

    /// Sets a new automata
    pub fn set_automata(&mut self, automata: Automata) {
        self.automata = Some(automata);
    }
}

impl ActiveKnowledgeBase for FakeActiveKnowledgeBase {
    fn start_target(&mut self) -> Result<(), String> {
        self.target_running = true;
        Ok(())
    }

    fn stop_target(&mut self) -> Result<(), String> {
        self.target_running = false;
        Ok(())
    }

    fn submit_word(&mut self, word: &Word) -> Result<Word, String> {
        if self.automata.is_none() {
            return Err("Automata cannot be None".to_string());
        }

        let automata = self.automata.as_ref().unwrap();
        let mut current_state = automata.initial_state.clone();
        let mut output_letters = Vec::new();

        for letter in word.letters() {
            match current_state.visit(letter) {
                Some((output_letter, next_state)) => {
                    output_letters.push(output_letter.clone());
                    current_state = next_state.clone();
                }
                None => {
                    output_letters.push(Letter::new(""));
                }
            }
        }

        Ok(Word::from_letters(output_letters))
    }

    fn is_target_running(&self) -> bool {
        self.target_running
    }
}

impl KnowledgeBaseTrait for FakeActiveKnowledgeBase {
    fn resolve_query(&mut self, query: &mut OutputQuery) -> Result<(), String> {
        match self.base.resolve_query(query) {
            Ok(_) => Ok(()),
            Err(_) => {
                self.start_target()?;
                let submit_result = self.submit_word(&query.input_word);
                let stop_result = self.stop_target();

                let output = submit_result?;
                stop_result?;

                self.base.add_word(&query.input_word, &output)?;
                query.set_result(output);
                Ok(())
            }
        }
    }

    fn add_word(&mut self, input_word: &Word, output_word: &Word) -> Result<(), String> {
        self.base.add_word(input_word, output_word)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creation() {
        let automata = Automata::new(
            crate::automata::State::new("S0".to_string()),
            "test".to_string(),
        );
        let kb = FakeActiveKnowledgeBase::new(automata);
        assert!(!kb.is_target_running());
    }

    #[test]
    fn test_start_stop() {
        let automata = Automata::new(
            crate::automata::State::new("S0".to_string()),
            "test".to_string(),
        );
        let mut kb = FakeActiveKnowledgeBase::new(automata);

        assert!(!kb.is_target_running());
        kb.start_target().unwrap();
        assert!(kb.is_target_running());
        kb.stop_target().unwrap();
        assert!(!kb.is_target_running());
    }
}
