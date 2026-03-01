use crate::automata::{Automata, State, Transition};
use crate::knowledge_base::KnowledgeBaseTrait;
use crate::letter::Letter;
use crate::query::OutputQuery;
use crate::word::Word;
use indexmap::IndexMap;
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::{Arc, Mutex};

/// The Observation Table is the core data structure of the L* algorithm
/// It maintains:
/// - D: the set of distinguishing strings (suffixes)
/// - S: the set of short prefixes
/// - SA: the set of long prefixes
/// - ot_content: the table content mapping (prefix, suffix) -> output
#[allow(non_snake_case)]
pub struct ObservationTable {
    pub input_letters: Vec<Letter>,
    knowledge_base: Arc<Mutex<dyn KnowledgeBaseTrait>>,
    pub D: Vec<Word>,
    pub S: Vec<Word>,
    pub SA: Vec<Word>,
    pub ot_content: HashMap<Word, HashMap<Word, Option<Letter>>>,
    pub initialized: bool,
}

type RowSignature = Vec<Option<Letter>>;
type Inconsistency = (((Word, Word), Word), Word);

#[allow(non_snake_case)]
impl ObservationTable {
    /// Create a new observation table
    pub fn new(
        input_letters: Vec<Letter>,
        knowledge_base: Arc<Mutex<dyn KnowledgeBaseTrait>>,
    ) -> Self {
        let capacity = input_letters.len() * 4;
        ObservationTable {
            input_letters,
            knowledge_base,
            D: Vec::with_capacity(capacity),
            S: Vec::with_capacity(capacity * 2),
            SA: Vec::with_capacity(capacity * 4),
            ot_content: HashMap::with_capacity(capacity),
            initialized: false,
        }
    }

    /// Initialize the observation table with empty string in S and all input letters in D
    pub fn initialize(&mut self) -> Result<(), String> {
        if self.initialized {
            return Err("Observation table already initialized".into());
        }

        self.initialized = true;
        self.D.clear();
        self.S.clear();
        self.SA.clear();
        self.ot_content.clear();

        // Add each input letter to D
        for letter in self.input_letters.clone() {
            let word = Word::from_letters(vec![letter]);
            self.add_word_in_D(word)?;
        }

        // Add empty word to S
        let empty_word = Word::new();
        self.add_word_in_S(empty_word)?;

        Ok(())
    }

    /// Serialize the observation table to a string
    pub fn serialize(&self) -> String {
        let mut result = String::new();
        result.push_str("=== Observation Table ===\n");
        result.push_str(&format!("D: {:?}\n", self.D));
        result.push_str(&format!("S: {:?}\n", self.S));
        result.push_str(&format!("SA: {:?}\n", self.SA));
        result.push_str(&format!("ot: {:?}\n", self.ot_content));
        result.push_str("\n\n\n");
        result
    }

    /// Find an inconsistency in the observation table
    /// Returns: Some(((prefix1, prefix2), (suffix, distinguishing_word))) when
    /// a pair of equivalent prefixes in `S` behave differently after appending
    /// `suffix` for some distinguishing column in `D`.
    pub fn find_inconsistency(&self) -> Option<Inconsistency> {
        // group equivalent rows in S by their row content
        let mut s_with_same_rows: HashMap<RowSignature, Vec<Word>> = HashMap::new();
        for word_in_s in &self.S {
            s_with_same_rows
                .entry(self.get_row(word_in_s))
                .or_default()
                .push(word_in_s.clone());
        }

        // for each group of equivalent rows, check combinations for inconsistencies
        for (_k, eq_word_in_s) in s_with_same_rows.iter() {
            if eq_word_in_s.len() > 1 {
                for pair_eq_words_in_S in eq_word_in_s
                    .iter()
                    .cloned()
                    .tuple_combinations::<(Word, Word)>()
                {
                    if let Some((suffix, distinguishing)) =
                        self.is_prefixes_equivalent(pair_eq_words_in_S.clone())
                    {
                        return Some(((pair_eq_words_in_S, suffix), distinguishing));
                    }
                }
            }
        }

        None
    }

    /// Check whether two prefixes in S are still equivalent after appending any
    /// input letter. If not, return the input-letter (as a Word) and the word in
    /// D that exhibits the difference.
    fn is_prefixes_equivalent(&self, eq_words_in_s: (Word, Word)) -> Option<(Word, Word)> {
        for input_letter in &self.input_letters {
            let suffix = Word::from_letters(vec![input_letter.clone()]);
            let initial_suffixed = eq_words_in_s.0.concatenate(&suffix);
            let eq_suffixed = eq_words_in_s.1.concatenate(&suffix);

            // compare over all D columns
            for word_in_d in &self.D {
                let cell_map = self.ot_content.get(word_in_d);
                let v1 = cell_map
                    .and_then(|m| m.get(&initial_suffixed))
                    .and_then(|o| o.as_ref());
                let v2 = cell_map
                    .and_then(|m| m.get(&eq_suffixed))
                    .and_then(|o| o.as_ref());

                if v1 != v2 {
                    // return the suffix (single input letter) and the distinguishing word in D
                    return Some((suffix, word_in_d.clone()));
                }
            }
        }

        None
    }

    /// Add a counterexample to the observation table
    pub fn add_counterexample(
        &mut self,
        input_word: &Word,
        _output_word: &Word,
    ) -> Result<(), String> {
        // Add all prefixes of the counterexample to S
        for len_prefix in 1..=input_word.len() {
            let prefix_input = input_word.prefix(len_prefix);
            if !self.S.contains(&prefix_input) {
                if self.SA.contains(&prefix_input) {
                    self.remove_row(prefix_input.clone())?;
                }
                self.add_word_in_S(prefix_input)?;
            }
        }
        Ok(())
    }

    fn remove_row(&mut self, word: Word) -> Result<(), String> {
        // remove from S
        self.S.retain(|w| w != &word);
        // remove from SA
        self.SA.retain(|w| w != &word);
        // remove from observation table content
        for (_key, row_map) in self.ot_content.iter_mut() {
            row_map.remove(&word);
        }
        Ok(())
    }

    /// Make the observation table consistent
    pub fn make_consistent(
        &mut self,
        inconsistency: Inconsistency,
    ) -> Result<(), String> {
        let ((_pair, suffix), inconsistent_suffix) = inconsistency;

        // new column = suffix + inconsistent_suffix
        let new_col_word = suffix.concatenate(&inconsistent_suffix);
        if !self.D.contains(&new_col_word) {
            self.add_word_in_D(new_col_word)?;
        }

        Ok(())
    }

    /// Check if the observation table is closed
    pub fn is_closed(&self) -> bool {
        let rows_in_s: HashSet<RowSignature> = self.S.iter().map(|s| self.get_row(s)).collect();

        for sa in &self.SA {
            if !rows_in_s.contains(&self.get_row(sa)) {
                return false;
            }
        }
        true
    }

    /// Close the observation table by moving rows from SA to S
    pub fn close_table(&mut self) -> Result<(), String> {
        let mut to_move = Vec::new();
        let rows_in_s: HashSet<RowSignature> = self.S.iter().map(|s| self.get_row(s)).collect();

        for sa in &self.SA {
            if !rows_in_s.contains(&self.get_row(sa)) {
                to_move.push(sa.clone());
            }
        }

        for word in to_move {
            self.SA.retain(|w| w != &word);
            self.add_word_in_S(word)?;
        }

        Ok(())
    }

    /// Get a row in the observation table
    fn get_row(&self, row_name: &Word) -> Vec<Option<Letter>> {
        let mut row = Vec::with_capacity(self.D.len());
        for word_in_D in &self.D {
            let value = self
                .ot_content
                .get(word_in_D)
                .and_then(|cell_content| cell_content.get(row_name))
                .cloned()
                .unwrap_or(None);
            row.push(value);
        }
        row
    }

    fn add_word_in_D(&mut self, word: Word) -> Result<(), String> {
        if self.D.contains(&word) {
            return Err(format!("Word {:?} already in D", word));
        }

        if self.ot_content.contains_key(&word) {
            return Err(format!(
                "Word {:?} already in content of the observation table",
                word
            ));
        }

        self.D.push(word.clone());

        // Execute queries for all S and SA with this new distinguishing word
        let words_to_query: Vec<Word> = self.S.iter().chain(self.SA.iter()).cloned().collect();
        let mut cell_content = HashMap::with_capacity(words_to_query.len());
        
        for word_in_S_or_SA in &words_to_query {
            let query_input = word_in_S_or_SA.concatenate(&word);
            let mut query = OutputQuery::new(query_input);
            self.execute_query(&mut query)?;

            if let Some(output_word) = query.output_word() {
                if let Some(output_letter) = output_word.last_letter() {
                    cell_content.insert(word_in_S_or_SA.clone(), Some(output_letter.clone()));
                }
            }
        }
        
        self.ot_content.insert(word, cell_content);
        Ok(())
    }

    fn add_word_in_S(&mut self, word: Word) -> Result<(), String> {
        if self.S.contains(&word) {
            return Err(format!("Word {:?} already in S", word));
        }
        if self.SA.contains(&word) {
            return Err(format!("Word {:?} already in SA", word));
        }

        self.S.push(word.clone());

        // Execute queries for all distinguishing words
        let d_words: Vec<Word> = self.D.iter().cloned().collect();
        for word_in_D in &d_words {
            let query_input = word.concatenate(word_in_D);
            let mut query = OutputQuery::new(query_input);
            self.execute_query(&mut query)?;

            if let Some(output_word) = query.output_word() {
                if let Some(output_letter) = output_word.last_letter() {
                    self.ot_content
                        .entry(word_in_D.clone())
                        .or_default()
                        .insert(word.clone(), Some(output_letter.clone()));
                }
            }
        }

        // Add sa := word.input_letter for each input letter
        let input_letters: Vec<Letter> = self.input_letters.iter().cloned().collect();
        for input_letter in &input_letters {
            let new_sa = word.concatenate(&Word::from_letters(vec![input_letter.clone()]));
            if !self.S.contains(&new_sa) {
                self.add_word_in_SA(new_sa)?;
            }
        }

        Ok(())
    }

    fn add_word_in_SA(&mut self, word: Word) -> Result<(), String> {
        if self.SA.contains(&word) {
            return Err(format!("Word {:?} already in SA", word));
        }
        if self.S.contains(&word) {
            return Err(format!("Word {:?} already in S", word));
        }

        self.SA.push(word.clone());

        // Execute queries for all distinguishing words
        let d_words: Vec<Word> = self.D.iter().cloned().collect();
        for word_in_D in &d_words {
            let query_input = word.concatenate(word_in_D);
            let mut query = OutputQuery::new(query_input);
            self.execute_query(&mut query)?;

            if let Some(output_word) = query.output_word() {
                if let Some(output_letter) = output_word.last_letter() {
                    self.ot_content
                        .entry(word_in_D.clone())
                        .or_default()
                        .insert(word.clone(), Some(output_letter.clone()));
                }
            }
        }

        Ok(())
    }

    fn execute_query(&mut self, query: &mut OutputQuery) -> Result<(), String> {
        self.knowledge_base.lock().unwrap().resolve_query(query)
    }
    /// Check if the observation table is consistent
    pub fn is_consistent(&self) -> bool {
        self.find_inconsistency().is_none()
    }

    /// Build a hypothesis automaton from the current observation table
    pub fn build_hypothesis(&self) -> Result<Automata, String> {
        if !self.is_closed() {
            return Err("Observation table must be closed".into());
        }
        if !self.is_consistent() {
            return Err("Observation table must be consistent".into());
        }

        let mut transitions = Vec::new();
        let mut words_and_states: Vec<(Word, State)> = Vec::new();
        let mut long_state_name_to_states: HashMap<RowSignature, usize> = HashMap::new();

        // Find unique rows in S
        let mut unique_rows: IndexMap<RowSignature, Vec<Word>> = IndexMap::new();
        for word in &self.S {
            unique_rows
                .entry(self.get_row(word))
                .or_default()
                .push(word.clone());
        }

        // Create states
        let mut initial_state = None;
        for (idx, (row_key, words_in_s)) in unique_rows.iter().enumerate() {
            let state_name = idx.to_string();
            let state = State::new(state_name.clone());

            // Check if this is the initial state (contains empty word)
            if words_in_s.iter().any(|w| w.is_empty()) {
                if initial_state.is_some() {
                    return Err("Multiple initial states found".into());
                }
                initial_state = Some(idx);
            }

            words_and_states.push((words_in_s[0].clone(), state.clone()));
            long_state_name_to_states.insert(row_key.clone(), idx);
        }

        let initial_state_idx = initial_state.ok_or("No initial state found")?;

        // Create transitions
        let input_words: Vec<(Letter, Word)> = self
            .input_letters
            .iter()
            .cloned()
            .map(|letter| {
                let one_letter_word = Word::from_letters(vec![letter.clone()]);
                (letter, one_letter_word)
            })
            .collect();
        for i in 0..words_and_states.len() {
            let word = words_and_states[i].0.clone();
            let source_state_name = words_and_states[i].1.clone();

            for (input_letter, input_letter_word) in &input_words {
                let new_word = word.concatenate(input_letter_word);
                let output_row = self.get_row(&new_word);
                let output_state_idx = long_state_name_to_states
                    .get(&output_row)
                    .copied()
                    .ok_or_else(|| format!("Cannot find a state matching row: {:?}", output_row))?;

                let output_letter = self
                    .ot_content
                    .get(input_letter_word)
                    .and_then(|cell| cell.get(&word))
                    .and_then(|opt| opt.as_ref())
                    .cloned()
                    .ok_or_else(|| {
                        format!(
                            "Missing output letter for input '{}' from word '{}'",
                            input_letter, word
                        )
                    })?;

                let transition_name = format!("t{}", transitions.len());
                let transition = Transition::new(
                    transition_name,
                    source_state_name.clone(),
                    words_and_states[output_state_idx].1.clone(),
                    input_letter.clone(),
                    output_letter,
                );
                words_and_states[i].1.add_transition(transition.clone());
                transitions.push(transition);
            }
        }

        let mut automata = Automata::new(
            words_and_states[initial_state_idx].1.clone(),
            "Automata".to_string(),
        );
        automata.transitions = transitions;

        Ok(automata)
    }
}

impl fmt::Display for ObservationTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut matrix: Vec<Vec<String>> = Vec::new();

        // Header row
        let mut header_row = vec![String::new()];
        header_row.extend(self.D.iter().map(|d| d.to_string()));
        matrix.push(header_row.clone());

        // S rows
        for row_name in &self.S {
            let mut row = vec![row_name.to_string()];
            row.extend(
                self.get_row(row_name)
                    .iter()
                    .map(|w| w.as_ref().map(|l| l.to_string()).unwrap_or_default()),
            );
            matrix.push(row);
        }

        // Separator
        matrix.push(vec!["~~~".to_string(); header_row.len()]);

        // SA rows
        for row_name in &self.SA {
            let mut row = vec![row_name.to_string()];
            row.extend(
                self.get_row(row_name)
                    .iter()
                    .map(|w| w.as_ref().map(|l| l.to_string()).unwrap_or_default()),
            );
            matrix.push(row);
        }

        // Calculate column widths
        let col_count = matrix[0].len();
        let mut col_widths = vec![0; col_count];
        for row in &matrix {
            for (i, cell) in row.iter().enumerate() {
                col_widths[i] = col_widths[i].max(cell.len());
            }
        }

        // Format and write
        for row in &matrix {
            for (i, cell) in row.iter().enumerate() {
                if i > 0 {
                    write!(f, " | ")?;
                }
                write!(f, "{:width$}", cell, width = col_widths[i])?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::knowledge_base::KnowledgeBase;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_observation_table_creation() {
        let kb: Arc<Mutex<dyn crate::knowledge_base::KnowledgeBaseTrait>> =
            Arc::new(Mutex::new(KnowledgeBase::new()));
        let letters = vec![Letter::new("a"), Letter::new("b")];
        let ot = ObservationTable::new(letters, kb);
        assert!(!ot.initialized);
    }
}
