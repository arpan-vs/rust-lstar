use std::fmt;
use std::hash::{Hash, Hasher};

/// Represents a single letter/symbol in the alphabet.
#[derive(Clone)]
pub enum Letter {
    Single(String),
    Multiple(Vec<String>),
}

impl Letter {
    /// Create a new letter from a single symbol
    pub fn new<S: ToString>(symbol: S) -> Self {
        Letter::Single(symbol.to_string())
    }

    /// Create a letter from multiple symbols
    pub fn from_symbols(symbols: Vec<String>) -> Self {
        if symbols.len() == 1 {
            Letter::Single(symbols.into_iter().next().unwrap())
        } else {
            let mut sorted = symbols;
            sorted.sort();
            sorted.dedup();
            Letter::Multiple(sorted)
        }
    }

    /// Get the symbols represented by this letter
    pub fn symbols(&self) -> String {
        match self {
            Letter::Single(s) => s.clone(),
            Letter::Multiple(v) => v.join(","),
        }
    }

    /// Get a string representation of this letter
    pub fn name(&self) -> String {
        match self {
            Letter::Single(s) => format!("Letter('{}')", s),
            Letter::Multiple(v) => {
                format!(
                    "Letter({})",
                    v.iter()
                        .map(|s| format!("'{}'", s))
                        .collect::<Vec<_>>()
                        .join(",")
                )
            }
        }
    }

    /// Check if this is an empty letter
    pub fn is_empty(&self) -> bool {
        false
    }

    /// Deserialize a comma-separated string of letter names into a Letter
    pub fn deserialize(str_letters: &str, possible_letters: &[Letter]) -> Result<Letter, String> {
        let mut letters = Vec::new();
        for str_letter in str_letters.split(',') {
            let found = possible_letters
                .iter()
                .find(|l| l.name() == str_letter)
                .ok_or_else(|| format!("Cannot find any letter that fit with '{}'", str_letter))?;
            letters.push(found.clone());
        }

        if letters.len() == 1 {
            Ok(letters[0].clone())
        } else {
            let symbols: Vec<String> = letters
                .iter()
                .flat_map(|l| match l {
                    Letter::Single(s) => vec![s.clone()],
                    Letter::Multiple(v) => v.clone(),
                })
                .collect();
            Ok(Letter::from_symbols(symbols))
        }
    }
}

impl PartialEq for Letter {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Letter::Single(a), Letter::Single(b)) => a == b,
            (Letter::Multiple(a), Letter::Multiple(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for Letter {}

impl Hash for Letter {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Letter::Single(s) => {
                0u8.hash(state);
                s.hash(state);
            }
            Letter::Multiple(v) => {
                1u8.hash(state);
                v.hash(state);
            }
        }
    }
}

impl fmt::Display for Letter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl fmt::Debug for Letter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Represents an empty letter, used as the identity element for words
#[derive(Clone, PartialEq, Eq)]
pub struct EmptyLetter;

impl EmptyLetter {
    pub fn new() -> Self {
        EmptyLetter
    }
}

impl Default for EmptyLetter {
    fn default() -> Self {
        EmptyLetter::new()
    }
}

impl Hash for EmptyLetter {
    fn hash<H: Hasher>(&self, state: &mut H) {
        "EmptyLetter".hash(state);
    }
}

impl fmt::Display for EmptyLetter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "EmptyLetter")
    }
}

impl fmt::Debug for EmptyLetter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "EmptyLetter")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_letter_creation() {
        let letter = Letter::new("a");
        assert_eq!(letter.name(), "Letter('a')");
    }

    #[test]
    fn test_letter_equality() {
        let l1 = Letter::new("a");
        let l2 = Letter::new("a");
        let l3 = Letter::new("b");

        assert_eq!(l1, l2);
        assert_ne!(l1, l3);
    }

    #[test]
    fn test_empty_letter() {
        let empty = EmptyLetter::new();
        assert_eq!(empty.to_string(), "EmptyLetter");
    }
}
