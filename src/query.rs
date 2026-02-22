use crate::word::Word;

/// Represents a query to be executed against the system under learning (SUL)
#[derive(Clone, Debug)]
pub struct OutputQuery {
    pub input_word: Word,
    pub output_word: Option<Word>,
    pub queried: bool,
}

impl OutputQuery {
    /// Create a new output query
    pub fn new(input_word: Word) -> Self {
        OutputQuery {
            input_word,
            output_word: None,
            queried: false,
        }
    }

    /// Check if the query has been executed
    pub fn is_queried(&self) -> bool {
        self.queried
    }

    /// Set the result of the query
    pub fn set_result(&mut self, output_word: Word) {
        self.output_word = Some(output_word);
        self.queried = true;
    }

    /// Get the output word (if query has been executed)
    pub fn output_word(&self) -> Option<&Word> {
        self.output_word.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::letter::Letter;

    #[test]
    fn test_query_creation() {
        let input = Word::from_letters(vec![Letter::new("a")]);
        let query = OutputQuery::new(input);
        assert!(!query.is_queried());
    }

    #[test]
    fn test_query_execution() {
        let input = Word::from_letters(vec![Letter::new("a")]);
        let output = Word::from_letters(vec![Letter::new("0")]);
        let mut query = OutputQuery::new(input);
        query.set_result(output);
        assert!(query.is_queried());
    }
}
