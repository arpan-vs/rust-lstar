//! Random-walk equivalence-test implementation.
//!
//! This strategy samples hypothesis paths and compares observed outputs against
//! the system under learning.

use crate::automata::{Automata, State};
use crate::eqtest::{Counterexample, EquivalenceTest};
use crate::knowledge_base::KnowledgeBaseTrait;
use crate::letter::Letter;
use crate::query::OutputQuery;
use crate::word::Word;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// Random-walk based equivalence-test strategy.
pub struct RandomWalkMethod {
    knowledge_base: Arc<Mutex<dyn KnowledgeBaseTrait>>,
    #[allow(dead_code)]
    input_letters: Vec<Letter>,
    max_steps: usize,
    restart_probability: f64,
}

struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new() -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        SimpleRng { state: seed }
    }

    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(1664525).wrapping_add(1013904223);
        self.state
    }

    fn gen_f64(&mut self) -> f64 {
        (self.next() as f64) / (u64::MAX as f64)
    }

    fn gen_range(&mut self, min: usize, max: usize) -> usize {
        if max <= min {
            return min;
        }
        min + (self.next() as usize) % (max - min)
    }
}

impl RandomWalkMethod {
    /// Create a new random-walk equivalence tester.
    ///
    /// `max_steps` bounds the number of transitions explored, and
    /// `restart_probability` controls how often the walk restarts at the
    /// initial state.
    pub fn new(
        knowledge_base: Arc<Mutex<dyn KnowledgeBaseTrait>>,
        input_letters: Vec<Letter>,
        max_steps: usize,
        restart_probability: f64,
    ) -> Self {
        RandomWalkMethod {
            knowledge_base,
            input_letters,
            max_steps,
            restart_probability,
        }
    }

    fn build_flat_outgoing(
        hypothesis: &Automata,
    ) -> Option<HashMap<String, Vec<(String, Letter, Letter)>>> {
        let can_use_flat_transitions = !hypothesis.transitions.is_empty()
            && hypothesis
                .transitions
                .iter()
                .all(|transition| !transition.source_state.name.is_empty());
        if !can_use_flat_transitions {
            return None;
        }

        let mut outgoing: HashMap<String, Vec<(String, Letter, Letter)>> = HashMap::new();
        for transition in &hypothesis.transitions {
            outgoing
                .entry(transition.source_state.name.clone())
                .or_default()
                .push((
                    transition.output_state.name.clone(),
                    transition.input_letter.clone(),
                    transition.output_letter.clone(),
                ));
        }
        Some(outgoing)
    }

    fn check_equivalence(
        &self,
        input_word: &Word,
        expected_output: &Word,
    ) -> Option<Counterexample> {
        let mut query = OutputQuery::new(input_word.clone());
        if self
            .knowledge_base
            .lock()
            .unwrap()
            .resolve_query(&mut query)
            .is_err()
        {
            return None;
        }

        if let Some(observed) = query.output_word() {
            if observed != expected_output {
                return Some(Counterexample {
                    input_word: input_word.clone(),
                    output_word: observed.clone(),
                });
            }
        }

        None
    }

    fn walk(
        &self,
        current_state: &State,
        rng: &mut SimpleRng,
        flat_outgoing: Option<&HashMap<String, Vec<(String, Letter, Letter)>>>,
    ) -> Option<(State, Letter, Letter)> {
        if let Some(flat_outgoing) = flat_outgoing {
            let outgoing = flat_outgoing.get(&current_state.name)?;

            if outgoing.is_empty() {
                return None;
            }

            let idx = rng.gen_range(0, outgoing.len());
            let picked_transition = &outgoing[idx];
            return Some((
                State::new(picked_transition.0.clone()),
                picked_transition.1.clone(),
                picked_transition.2.clone(),
            ));
        }

        if current_state.transitions.is_empty() {
            return None;
        }

        let idx = rng.gen_range(0, current_state.transitions.len());
        let picked_transition = &current_state.transitions[idx];

        Some((
            picked_transition.output_state.clone(),
            picked_transition.input_letter.clone(),
            picked_transition.output_letter.clone(),
        ))
    }
}

impl EquivalenceTest for RandomWalkMethod {
    fn find_counterexample(&self, hypothesis: &mut Automata) -> Option<Counterexample> {
        let mut rng = SimpleRng::new();
        let flat_outgoing = Self::build_flat_outgoing(hypothesis);
        let mut i_step = 0;
        let mut first_step_after_restart = true;
        let mut current_state = hypothesis.initial_state.clone();
        let mut input_word = Word::new();
        let mut hypothesis_output_word = Word::new();
        let mut force_restart = false;

        while i_step < self.max_steps {
            if !first_step_after_restart {
                if force_restart || rng.gen_f64() < self.restart_probability {
                    current_state = hypothesis.initial_state.clone();
                    first_step_after_restart = true;

                    if let Some(ce) = self.check_equivalence(&input_word, &hypothesis_output_word) {
                        return Some(ce);
                    }

                    input_word = Word::new();
                    hypothesis_output_word = Word::new();
                    force_restart = false;
                }
            } else {
                first_step_after_restart = false;
            }

            match self.walk(&current_state, &mut rng, flat_outgoing.as_ref()) {
                Some((new_state, input_letter, output_letter)) => {
                    current_state = new_state;
                    input_word = input_word.append_letter(input_letter);
                    hypothesis_output_word = hypothesis_output_word.append_letter(output_letter);
                }
                None => {
                    force_restart = true;
                }
            }

            i_step += 1;
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::automata::{Automata, State, Transition};
    use crate::knowledge_base::KnowledgeBaseTrait;
    use std::sync::{Arc, Mutex};

    struct ConstantOutputKb {
        output_symbol: String,
    }

    impl ConstantOutputKb {
        fn new(output_symbol: &str) -> Self {
            Self {
                output_symbol: output_symbol.to_string(),
            }
        }
    }

    impl KnowledgeBaseTrait for ConstantOutputKb {
        fn resolve_query(&mut self, query: &mut OutputQuery) -> Result<(), String> {
            let outputs = query
                .input_word
                .letters()
                .iter()
                .map(|_| Letter::new(self.output_symbol.clone()))
                .collect::<Vec<_>>();
            query.set_result(Word::from_letters(outputs));
            Ok(())
        }

        fn add_word(&mut self, _input_word: &Word, _output_word: &Word) -> Result<(), String> {
            Ok(())
        }
    }

    fn build_flat_single_state_automata(output_symbol: &str) -> Automata {
        let initial = State::new("0".to_string());
        let mut automata = Automata::new(initial, "A".to_string());
        automata.transitions = vec![Transition::new(
            "t0".to_string(),
            State::new("0".to_string()),
            State::new("0".to_string()),
            Letter::new("a"),
            Letter::new(output_symbol),
        )];
        automata
    }

    #[test]
    fn random_walk_finds_counterexample_on_flat_transition_automata() {
        let kb: Arc<Mutex<dyn KnowledgeBaseTrait>> =
            Arc::new(Mutex::new(ConstantOutputKb::new("1")));
        let eq = RandomWalkMethod::new(kb, vec![Letter::new("a")], 16, 1.0);
        let mut hypothesis = build_flat_single_state_automata("0");

        let ce = eq.find_counterexample(&mut hypothesis);
        assert!(ce.is_some());
    }

    #[test]
    fn random_walk_returns_none_when_outputs_match() {
        let kb: Arc<Mutex<dyn KnowledgeBaseTrait>> =
            Arc::new(Mutex::new(ConstantOutputKb::new("0")));
        let eq = RandomWalkMethod::new(kb, vec![Letter::new("a")], 16, 1.0);
        let mut hypothesis = build_flat_single_state_automata("0");

        let ce = eq.find_counterexample(&mut hypothesis);
        assert!(ce.is_none());
    }
}
