use super::state::State;
use super::transition::Transition;
use crate::letter::Letter;
use crate::Word;
use std::collections::{HashMap, HashSet};
use std::fmt;

/// Represents a complete automaton
#[derive(Clone, Debug)]
pub struct Automata {
    pub initial_state: State,
    pub transitions: Vec<Transition>,
    pub name: String,
    transition_index: Option<HashMap<String, HashMap<Letter, usize>>>,
}

impl Automata {
    fn can_use_flat_transitions(&self) -> bool {
        !self.transitions.is_empty()
            && self
                .transitions
                .iter()
                .all(|transition| !transition.source_state.is_empty())
    }

    pub fn new(initial_state: State, name: String) -> Self {
        Automata {
            initial_state: initial_state.clone(),
            transitions: Vec::new(),
            name,
            transition_index: None,
        }
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    /// Set initial state to the automaton
    pub fn set_initial_state(&mut self, state: State) {
        self.initial_state = state;
    }

    /// Get the initial state of the automaton
    pub fn get_initial_state(&self) -> &State {
        &self.initial_state
    }

    pub fn parse_dot(input: &str) -> Result<Automata, String> {
        super::dot_parser::parse_dot(input)
    }
    /// Build DOT code representation
    pub fn build_dot_code(&self) -> String {
        // Delegate DOT generation to the DOT parser's builder (keeps logic centralized)
        super::dot_parser::build_dot_code(self)
    }

    /// Build transition index for fast lookups
    fn build_transition_index(&mut self) {
        if self.transition_index.is_some() {
            return;
        }
        let mut index = HashMap::with_capacity(self.transitions.len());
        for (i, transition) in self.transitions.iter().enumerate() {
            index
                .entry(transition.source_state.clone())
                .or_insert_with(HashMap::new)
                .insert(transition.input_letter.clone(), i);
        }
        self.transition_index = Some(index);
    }

    /// Get all reachable states from initial state
    pub fn get_states(&self) -> Vec<State> {
        if self.can_use_flat_transitions() {
            let mut seen: HashSet<String> = HashSet::new();
            let mut state_names: Vec<String> = Vec::new();

            if seen.insert(self.initial_state.name.clone()) {
                state_names.push(self.initial_state.name.clone());
            }

            for transition in &self.transitions {
                if seen.insert(transition.source_state.clone()) {
                    state_names.push(transition.source_state.clone());
                }
                if seen.insert(transition.output_state.name.clone()) {
                    state_names.push(transition.output_state.name.clone());
                }
            }

            return state_names.into_iter().map(State::new).collect();
        }

        // Visits the automata to discover all the available states.
        let mut states: Vec<State> = Vec::new();
        let mut to_analyze: Vec<State> = Vec::new();

        to_analyze.push(self.initial_state.clone());

        while let Some(current_state) = to_analyze.pop() {
            if !states.contains(&current_state) {
                for tr in &current_state.transitions {
                    let next_state = tr.output_state.clone();
                    if !states.contains(&next_state) && !to_analyze.contains(&next_state) {
                        to_analyze.push(next_state);
                    }
                }
                states.push(current_state.clone());
            }
        }
        states
    }

    /// Play a word through the automaton starting from a given state
    pub fn play_word(
        &mut self,
        input_word: &Word,
        starting_state: Option<&State>,
    ) -> Result<(Word, Vec<State>), String> {
        if input_word.is_empty() {
            return Err("Input word cannot be None or empty".to_string());
        }

        if self.can_use_flat_transitions() {
            self.build_transition_index();
            let transition_index = self.transition_index.as_ref().unwrap();

            let mut current_state_name = starting_state
                .map(|s| s.name.clone())
                .unwrap_or_else(|| self.initial_state.name.clone());

            let mut output_letters: Vec<Letter> = Vec::with_capacity(input_word.len());
            let mut visited_states: Vec<State> = Vec::with_capacity(input_word.len());

            for letter in input_word.letters() {
                let transition_idx = transition_index
                    .get(current_state_name.as_str())
                    .and_then(|outgoing| outgoing.get(letter))
                    .ok_or_else(|| {
                        format!(
                            "No transition found from state '{}' for letter {:?}",
                            current_state_name, letter
                        )
                    })?;

                let transition = &self.transitions[*transition_idx];
                output_letters.push(transition.output_letter.clone());
                visited_states.push(State::new(transition.output_state.name.clone()));
                current_state_name = transition.output_state.name.clone();
            }

            let output_word = Word::from_letters(output_letters);
            return Ok((output_word, visited_states));
        }

        let mut current_state = &self.initial_state;

        if let Some(start_state) = starting_state {
            current_state = start_state;
        }

        let mut output_letters: Vec<Letter> = Vec::new();
        let mut visited_states: Vec<State> = Vec::new();

        for letter in input_word.letters() {
            let (output_letter, output_state) = current_state
                .visit(letter)
                .ok_or_else(|| format!("No transition found for letter {:?}", letter))?;

            output_letters.push(output_letter.clone());
            visited_states.push(State::new(output_state.name.clone()));

            current_state = output_state;
        }

        let output_word = Word::from_letters(output_letters);
        Ok((output_word, visited_states))
    }

    /// Play an `OutputQuery`'s input word on this automaton and return
    /// the produced output word and visited states (if any).
    pub fn play_query(
        &mut self,
        query: &crate::query::OutputQuery,
    ) -> Result<(Word, Vec<State>), String> {
        let initial_state = self.initial_state.clone();
        self.play_word(&query.input_word, Some(&initial_state))
    }
}

impl fmt::Display for Automata {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.build_dot_code())
    }
}
