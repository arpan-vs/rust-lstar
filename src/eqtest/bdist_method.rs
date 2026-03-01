//! BDist equivalence-test implementation.
//!
//! The strategy compares state representatives and searches distinguishing
//! suffixes up to a bounded depth.

use crate::automata::Automata;
use crate::eqtest::{Counterexample, EquivalenceTest};
use crate::knowledge_base::KnowledgeBaseTrait;
use crate::letter::Letter;
use crate::query::OutputQuery;
use crate::word::Word;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

/// BDist equivalence-test strategy.
pub struct BDistMethod {
    knowledge_base: Arc<Mutex<dyn KnowledgeBaseTrait>>,
    input_letters: Vec<Letter>,
    bdist: usize,
}

impl BDistMethod {
    /// Create a new BDist equivalence tester.
    ///
    /// `bdist` controls the maximum distinguishing suffix length.
    pub fn new(
        knowledge_base: Arc<Mutex<dyn KnowledgeBaseTrait>>,
        input_letters: Vec<Letter>,
        bdist: usize,
    ) -> Self {
        Self {
            knowledge_base,
            input_letters,
            bdist,
        }
    }

    fn can_use_flat_transitions(hypothesis: &Automata) -> bool {
        !hypothesis.transitions.is_empty()
            && hypothesis
                .transitions
                .iter()
                .all(|transition| !transition.source_state.name.is_empty())
    }

    fn outgoing_inputs(&self, hypothesis: &Automata, state_name: &str) -> Vec<(Letter, String)> {
        if Self::can_use_flat_transitions(hypothesis) {
            return hypothesis
                .transitions
                .iter()
                .filter(|t| t.source_state.name == state_name)
                .map(|t| (t.input_letter.clone(), t.output_state.name.clone()))
                .collect();
        }

        let states = hypothesis.get_states();
        if let Some(state) = states.iter().find(|s| s.name == state_name) {
            return state
                .transitions
                .iter()
                .map(|t| (t.input_letter.clone(), t.output_state.name.clone()))
                .collect();
        }
        Vec::new()
    }

    fn compute_representatives(&self, hypothesis: &Automata) -> HashMap<String, Word> {
        let mut representatives = HashMap::new();
        let mut queue = VecDeque::new();
        let initial = hypothesis.initial_state.name.clone();
        representatives.insert(initial.clone(), Word::new());
        queue.push_back(initial);

        while let Some(state_name) = queue.pop_front() {
            let rep_word = match representatives.get(&state_name) {
                Some(w) => w.clone(),
                None => continue,
            };

            for (input_letter, next_state_name) in self.outgoing_inputs(hypothesis, &state_name) {
                if next_state_name == state_name || representatives.contains_key(&next_state_name) {
                    continue;
                }
                let candidate = rep_word.concatenate(&Word::from_letters(vec![input_letter]));
                representatives.insert(next_state_name.clone(), candidate);
                queue.push_back(next_state_name);
            }
        }

        representatives
    }

    fn generate_suffixes(&self) -> Vec<Word> {
        fn generate_len(
            letters: &[Letter],
            len: usize,
            current: &mut Vec<Letter>,
            acc: &mut Vec<Word>,
        ) {
            if current.len() == len {
                acc.push(Word::from_letters(current.clone()));
                return;
            }
            for letter in letters {
                current.push(letter.clone());
                generate_len(letters, len, current, acc);
                current.pop();
            }
        }

        let mut suffixes = Vec::new();
        for len in 1..=self.bdist {
            let mut current = Vec::new();
            generate_len(&self.input_letters, len, &mut current, &mut suffixes);
        }
        suffixes
    }

    fn check_equivalence(
        &self,
        w_i: &Word,
        w_i_prime: &Word,
        suffixes: &[Word],
    ) -> Option<(Word, OutputQuery, OutputQuery)> {
        for suffix in suffixes {
            let mut query_i = OutputQuery::new(w_i.concatenate(suffix));
            let mut query_i_prime = OutputQuery::new(w_i_prime.concatenate(suffix));

            if self
                .knowledge_base
                .lock()
                .unwrap()
                .resolve_query(&mut query_i)
                .is_err()
            {
                continue;
            }
            if self
                .knowledge_base
                .lock()
                .unwrap()
                .resolve_query(&mut query_i_prime)
                .is_err()
            {
                continue;
            }

            let last_i = query_i.output_word().and_then(|w| w.last_letter()).cloned();
            let last_i_prime = query_i_prime
                .output_word()
                .and_then(|w| w.last_letter())
                .cloned();

            if last_i != last_i_prime {
                return Some((suffix.clone(), query_i, query_i_prime));
            }
        }
        None
    }
}

impl EquivalenceTest for BDistMethod {
    fn find_counterexample(&self, hypothesis: &mut Automata) -> Option<Counterexample> {
        let representatives = self.compute_representatives(hypothesis);
        let suffixes = self.generate_suffixes();
        let states = hypothesis.get_states();

        for state in &states {
            for letter in &self.input_letters {
                let rep = match representatives.get(&state.name) {
                    Some(w) => w.clone(),
                    None => continue,
                };
                let w_i = rep.concatenate(&Word::from_letters(vec![letter.clone()]));
                let one_letter_word = Word::from_letters(vec![letter.clone()]);

                let (hyp_output_word, visited_states) =
                    match hypothesis.play_word(&one_letter_word, Some(state)) {
                        Ok(v) => v,
                        Err(_) => continue,
                    };

                let mut query = OutputQuery::new(w_i.clone());
                if self
                    .knowledge_base
                    .lock()
                    .unwrap()
                    .resolve_query(&mut query)
                    .is_err()
                {
                    continue;
                }

                let hyp_last = hyp_output_word.last_letter().cloned();
                let real_last = query.output_word().and_then(|w| w.last_letter()).cloned();
                if hyp_last != real_last {
                    return query
                        .output_word()
                        .cloned()
                        .map(|output_word| Counterexample {
                            input_word: query.input_word.clone(),
                            output_word,
                        });
                }

                let q_prime_name = match visited_states.last() {
                    Some(s) => s.name.clone(),
                    None => continue,
                };
                let w_i_prime = match representatives.get(&q_prime_name) {
                    Some(w) => w,
                    None => continue,
                };
                if &w_i == w_i_prime {
                    continue;
                }

                if let Some((suffix, query_i, query_i_prime)) =
                    self.check_equivalence(&w_i, w_i_prime, &suffixes)
                {
                    let w_i_suffix = w_i.concatenate(&suffix);
                    let expected_i = match hypothesis.play_word(&w_i_suffix, None) {
                        Ok((w, _)) => w,
                        Err(_) => continue,
                    };
                    let observed_i = match query_i.output_word() {
                        Some(w) => w.clone(),
                        None => continue,
                    };

                    let chosen_query = if expected_i != observed_i {
                        query_i
                    } else {
                        query_i_prime
                    };

                    if let Some(output_word) = chosen_query.output_word().cloned() {
                        return Some(Counterexample {
                            input_word: chosen_query.input_word.clone(),
                            output_word,
                        });
                    }
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::automata::{State, Transition};

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

    fn build_single_state_hypothesis(output_symbol: &str) -> Automata {
        let mut s0 = State::new("S0".to_string());
        s0.add_transition(Transition::new(
            "t0".to_string(),
            State::new("S0".to_string()),
            State::new("S0".to_string()),
            Letter::new("a"),
            Letter::new(output_symbol),
        ));
        Automata::new(s0, "H".to_string())
    }

    #[test]
    fn bdist_finds_counterexample_for_wrong_hypothesis() {
        let kb: Arc<Mutex<dyn KnowledgeBaseTrait>> =
            Arc::new(Mutex::new(ConstantOutputKb::new("1")));
        let eq = BDistMethod::new(kb, vec![Letter::new("a")], 2);
        let mut hypothesis = build_single_state_hypothesis("0");
        assert!(eq.find_counterexample(&mut hypothesis).is_some());
    }

    #[test]
    fn bdist_returns_none_for_equivalent_hypothesis() {
        let kb: Arc<Mutex<dyn KnowledgeBaseTrait>> =
            Arc::new(Mutex::new(ConstantOutputKb::new("0")));
        let eq = BDistMethod::new(kb, vec![Letter::new("a")], 2);
        let mut hypothesis = build_single_state_hypothesis("0");
        assert!(eq.find_counterexample(&mut hypothesis).is_none());
    }
}
