use crate::letter::Letter;
use smallvec::SmallVec;
use std::fmt;
use std::hash::{Hash, Hasher};

/// Represents a word - a sequence of letters
#[derive(Clone, Debug)]
pub struct Word {
    letters: SmallVec<[Letter; 8]>,
}

impl Word {
    /// Create a new empty word
    pub fn new() -> Self {
        Word {
            letters: SmallVec::new(),
        }
    }

    /// Create a word from a vector of letters
    pub fn from_letters(letters: Vec<Letter>) -> Self {
        Word { letters: SmallVec::from_vec(letters) }
    }

    /// Get the letters in this word
    pub fn letters(&self) -> &[Letter] {
        &self.letters
    }

    /// Get the length of this word
    pub fn len(&self) -> usize {
        self.letters.len()
    }

    /// Check if the word is empty
    pub fn is_empty(&self) -> bool {
        self.letters.is_empty()
    }

    /// Get the last letter
    pub fn last_letter(&self) -> Option<&Letter> {
        self.letters.last()
    }

    /// Append a letter to this word
    pub fn append_letter(mut self, letter: Letter) -> Self {
        self.letters.push(letter);
        self
    }

    /// Concatenate two words
    pub fn concatenate(&self, other: &Word) -> Word {
        let mut letters = SmallVec::with_capacity(self.letters.len() + other.letters.len());
        letters.extend(self.letters.iter().cloned());
        letters.extend(other.letters.iter().cloned());
        Word { letters }
    }

    /// Concatenate with a single letter (optimized)
    pub fn concatenate_letter(&self, letter: &Letter) -> Word {
        let mut letters = SmallVec::with_capacity(self.letters.len() + 1);
        letters.extend(self.letters.iter().cloned());
        letters.push(letter.clone());
        Word { letters }
    }

    /// Get a prefix of the word up to (but not including) the given length
    pub fn prefix(&self, len: usize) -> Word {
        let prefix_len = std::cmp::min(len, self.letters.len());
        Word {
            letters: self.letters[..prefix_len].iter().cloned().collect(),
        }
    }
}

impl Default for Word {
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for Word {
    fn eq(&self, other: &Self) -> bool {
        self.letters == other.letters
    }
}

impl Eq for Word {}

impl Hash for Word {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.letters.hash(state);
    }
}

impl fmt::Display for Word {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[{}]",
            self.letters
                .iter()
                .map(|l| l.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word_creation() {
        let word = Word::new();
        assert!(word.is_empty());
        assert_eq!(word.len(), 0);
    }

    #[test]
    fn test_word_from_letters() {
        let letters = vec![Letter::new("a"), Letter::new("b")];
        let word = Word::from_letters(letters);
        assert_eq!(word.len(), 2);
    }

    #[test]
    fn test_word_append() {
        let word = Word::new();
        let word = word.append_letter(Letter::new("a"));
        assert_eq!(word.len(), 1);
    }

    #[test]
    fn test_word_concatenate() {
        let w1 = Word::from_letters(vec![Letter::new("a")]);
        let w2 = Word::from_letters(vec![Letter::new("b")]);
        let w3 = w1.concatenate(&w2);
        assert_eq!(w3.len(), 2);
    }

    #[test]
    fn test_word_prefix() {
        let word = Word::from_letters(vec![Letter::new("a"), Letter::new("b"), Letter::new("c")]);
        let prefix = word.prefix(2);
        assert_eq!(prefix.len(), 2);
    }
}
