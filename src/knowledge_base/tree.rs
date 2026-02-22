/// Knowledge Tree Implementation
///
/// A tree-based structure that stores the relationship between input words and output words.
/// This implementation is based on the pylstar KnowledgeTree used for caching query results.
use crate::letter::Letter;
use crate::word::Word;
use core::fmt;
use std::collections::HashMap;

/// A node in the knowledge tree
#[derive(Clone, Debug)]
pub struct KnowledgeNode {
    input_letter: Letter,
    output_letter: Letter,
    children: HashMap<Letter, KnowledgeNode>,
}

impl KnowledgeNode {
    /// Create a new knowledge node
    pub fn new(input_letter: Letter, output_letter: Letter) -> Self {
        KnowledgeNode {
            input_letter,
            output_letter,
            children: HashMap::new(),
        }
    }

    /// Get the input letter of this node
    pub fn input_letter(&self) -> &Letter {
        &self.input_letter
    }

    /// Get the output letter of this node
    pub fn output_letter(&self) -> &Letter {
        &self.output_letter
    }

    /// Get the children of this node
    pub fn children(&self) -> &HashMap<Letter, KnowledgeNode> {
        &self.children
    }

    pub fn serialize(&self) -> HashMap<String, String> {
        let mut node = HashMap::new();
        node.insert("input_letter".to_string(), self.input_letter.symbols());
        node.insert("output_letter".to_string(), self.output_letter.symbols());
        let children: Vec<_> = self.children.iter().map(|(_k, v)| v.serialize()).collect();
        node.insert("children".to_string(), format!("{:?}", children));
        node
    }

    pub fn deserialize(
        dict_data: &HashMap<String, String>,
        possible_letters: &[Letter],
    ) -> Result<KnowledgeNode, String> {
        let input_letter = Letter::deserialize(
            dict_data
                .get("input_letter")
                .ok_or("Missing input_letter")?,
            possible_letters,
        )?;
        let output_letter = Letter::deserialize(
            dict_data
                .get("output_letter")
                .ok_or("Missing output_letter")?,
            possible_letters,
        )?;
        let mut node = KnowledgeNode::new(input_letter, output_letter);

        if let Some(children_str) = dict_data.get("children") {
            if let Ok(children) = serde_json::from_str::<Vec<HashMap<String, String>>>(children_str)
            {
                for child_map in children {
                    let child_node = KnowledgeNode::deserialize(&child_map, possible_letters)?;
                    node.children
                        .insert(child_node.input_letter.clone(), child_node);
                }
            }
        }

        Ok(node)
    }

    pub fn traverse(
        &mut self,
        input_letters: &[Letter],
        output_letters: Option<&[Letter]>,
    ) -> Result<Vec<Letter>, String> {
        if input_letters[0] != self.input_letter {
            return Err(format!(
                "Node cannot be traversed with input letter '{}'",
                input_letters[0]
            ));
        }
        if let Some(output_letters) = output_letters {
            if output_letters[0] != self.output_letter {
                return Err(format!(
                    "Node '{}' cannot be traversed with output letter '{}'",
                    self.input_letter, output_letters[0]
                ));
            }
            if input_letters.len() != output_letters.len() {
                return Err(
                    "Specified input and output letters do not have the same length".to_string(),
                );
            }
        }

        if input_letters.len() < 2 {
            return Ok(vec![self.output_letter.clone()]);
        }

        let current_input_letter = &input_letters[1];
        let current_output_letter = output_letters.map(|ol| &ol[1]);

        if let Some(child) = self.children.get_mut(current_input_letter) {
            if let Some(current_output) = current_output_letter {
                if child.output_letter != *current_output {
                    return Err(format!(
                        "Incompatible path found, expected '{}' found '{}'",
                        child.output_letter.symbols(),
                        current_output.symbols()
                    ));
                }
            }

            let new_output_letters = output_letters.map(|ol| &ol[1..]);
            let new_input_letters = &input_letters[1..];

            let mut result = vec![self.output_letter.clone()];
            result.extend(child.traverse(new_input_letters, new_output_letters)?);
            Ok(result)
        } else if output_letters.is_some() {
            let mut new_child =
                KnowledgeNode::new(input_letters[1].clone(), output_letters.unwrap()[1].clone());
            let new_input_letters = &input_letters[1..];
            let new_output_letters = &output_letters.unwrap()[1..];

            let mut result = vec![self.output_letter.clone()];
            result.extend(new_child.traverse(new_input_letters, Some(new_output_letters))?);

            self.children
                .insert(new_child.input_letter.clone(), new_child);
            Ok(result)
        } else {
            let letters_str: Vec<String> = input_letters.iter().map(|l| l.to_string()).collect();
            Err(format!(
                "Cannot traverse node '{}' with subsequences '{}'",
                self.input_letter,
                letters_str.join(", ")
            ))
        }
    }
}

impl fmt::Display for KnowledgeNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let serialized = self.serialize();
        match serde_json::to_string_pretty(&serialized) {
            Ok(json) => write!(f, "{}", json),
            Err(_) => write!(f, "{:?}", serialized),
        }
    }
}

/// A tree that stores the relationship between input and output words
#[derive(Clone, Debug)]
pub struct KnowledgeTree {
    roots: Vec<KnowledgeNode>,
    nb_added_words: usize,
}

impl KnowledgeTree {
    /// Create a new empty knowledge tree
    pub fn new() -> Self {
        KnowledgeTree {
            roots: Vec::new(),
            nb_added_words: 0,
        }
    }

    /// Get the roots of the tree
    pub fn roots(&self) -> &Vec<KnowledgeNode> {
        &self.roots
    }

    /// Get the number of words added
    pub fn num_added_words(&self) -> usize {
        self.nb_added_words
    }

    /// Get the output word for a given input word
    ///
    /// Returns an error if no path exists in the tree for the input.
    pub fn get_output_word(&mut self, input_word: &Word) -> Result<Word, String> {
        for root in &mut self.roots {
            if let Ok(output_letters) = root.traverse(input_word.letters(), None) {
                return Ok(Word::from_letters(output_letters));
            }
        }
        Err("No path found".to_string())
    }

    /// Add a word mapping to the tree
    ///
    /// Creates or traverses the tree to establish the relationship between
    /// the input and output words.
    pub fn add_word(&mut self, input_word: &Word, output_word: &Word) -> Result<(), String> {
        if input_word.len() != output_word.len() {
            return Err("Input and output words do not have the same size".to_string());
        }
        self.add_letters(input_word.letters(), output_word.letters())?;
        self.nb_added_words += 1;
        Ok(())
    }

    /// Internal method to add letters to the tree
    fn add_letters(
        &mut self,
        input_letters: &[Letter],
        output_letters: &[Letter],
    ) -> Result<(), String> {
        let mut retained_root: Option<&mut KnowledgeNode> = None;

        for root in &mut self.roots {
            if root.input_letter == input_letters[0] {
                if root.output_letter != output_letters[0] {
                    return Err(format!(
                        "Incompatible path found, expected '{}' found '{}'",
                        root.output_letter.symbols(),
                        output_letters[0].symbols()
                    ));
                }
                retained_root = Some(root);
                break;
            }
        }

        let root = if let Some(root) = retained_root {
            root
        } else {
            let new_root = KnowledgeNode::new(input_letters[0].clone(), output_letters[0].clone());
            self.roots.push(new_root);
            self.roots.last_mut().unwrap()
        };

        root.traverse(input_letters, Some(output_letters))?;
        Ok(())
    }
}

impl Default for KnowledgeTree {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_retrieve_word() {
        let mut tree = KnowledgeTree::new();
        let input = Word::from_letters(vec![Letter::new("a"), Letter::new("b")]);
        let output = Word::from_letters(vec![Letter::new(1), Letter::new(2)]);

        tree.add_word(&input, &output).unwrap();

        let retrieved = tree.get_output_word(&input).unwrap();
        assert_eq!(retrieved, output);
    }

    #[test]
    fn test_multiple_words() {
        let mut tree = KnowledgeTree::new();

        let input1 = Word::from_letters(vec![Letter::new("a"), Letter::new("b")]);
        let output1 = Word::from_letters(vec![Letter::new(1), Letter::new(2)]);

        let input2 = Word::from_letters(vec![Letter::new("a"), Letter::new("c")]);
        let output2 = Word::from_letters(vec![Letter::new(1), Letter::new(3)]);

        tree.add_word(&input1, &output1).unwrap();
        tree.add_word(&input2, &output2).unwrap();

        assert_eq!(tree.get_output_word(&input1).unwrap(), output1);
        assert_eq!(tree.get_output_word(&input2).unwrap(), output2);
    }

    #[test]
    fn test_incompatible_path_error() {
        let mut tree = KnowledgeTree::new();

        let input1 = Word::from_letters(vec![Letter::new("a"), Letter::new("b")]);
        let output1 = Word::from_letters(vec![Letter::new(1), Letter::new(2)]);

        let input2 = Word::from_letters(vec![Letter::new("a"), Letter::new("b")]);
        let output2 = Word::from_letters(vec![Letter::new(1), Letter::new(3)]);

        tree.add_word(&input1, &output1).unwrap();

        // This should fail because the path already exists with different output
        let result = tree.add_word(&input2, &output2);
        assert!(result.is_err());
    }

    #[test]
    fn test_retrieve_nonexistent_word() {
        let mut tree = KnowledgeTree::new();
        let input = Word::from_letters(vec![Letter::new("x"), Letter::new("y")]);

        let result = tree.get_output_word(&input);
        assert!(result.is_err());
    }
}
