//! W-method equivalence-test implementation.
//!
//! Generates test suites from transition-cover and characterization sets.

use crate::automata::{Automata, State};
use crate::eqtest::{Counterexample, EquivalenceTest};
use crate::knowledge_base::KnowledgeBaseTrait;
use crate::letter::Letter;
use crate::query::OutputQuery;
use crate::word::Word;
use std::collections::{HashSet, VecDeque};
use std::sync::{Arc, Mutex};

/// W-method equivalence-test strategy.
pub struct WMethodEQ {
    knowledge_base: Arc<Mutex<dyn KnowledgeBaseTrait>>,
    input_letters: Vec<Letter>,
    /// Upper bound on the number of states in the target system.
    pub max_states: usize,
}

impl WMethodEQ {
    /// Create a new W-method equivalence tester.
    pub fn new(
        knowledge_base: Arc<Mutex<dyn KnowledgeBaseTrait>>,
        input_letters: Vec<Letter>,
        max_states: usize,
    ) -> Self {
        WMethodEQ {
            knowledge_base,
            input_letters,
            max_states,
        }
    }

    fn compute_distinguishable_string(
        &self,
        hypothesis: &mut Automata,
        couple: (&State, &State),
    ) -> OutputQuery {
        let mut queries_to_test = VecDeque::new();
        let empty_word = Word::new();
        let z_query = OutputQuery::new(empty_word.clone());

        for letter in &self.input_letters {
            let new_word = empty_word.concatenate(&Word::from_letters(vec![letter.clone()]));
            queries_to_test.push_back(OutputQuery::new(new_word));
        }

        let mut distinguishable_query = z_query;
        let mut i = 0;

        while let Some(query) = queries_to_test.pop_front() {
            if i > self.max_states * self.max_states {
                break;
            }

            if self.is_distinguishable_states(hypothesis, &query, couple) {
                distinguishable_query = query;
                break;
            } else {
                for letter in &self.input_letters {
                    let new_query = OutputQuery::new(
                        query
                            .input_word
                            .concatenate(&Word::from_letters(vec![letter.clone()])),
                    );
                    queries_to_test.push_back(new_query);
                }
            }
            i += 1;
        }

        distinguishable_query
    }

    fn is_distinguishable_states(
        &self,
        hypothesis: &mut Automata,
        query: &OutputQuery,
        couple: (&State, &State),
    ) -> bool {
        let output0 = hypothesis.play_word(&query.input_word, Some(couple.0));
        let output1 = hypothesis.play_word(&query.input_word, Some(couple.1));

        output0 != output1
    }

    fn compute_p(&self, hypothesis: &mut Automata) -> Vec<OutputQuery> {
        let mut p = Vec::new();
        let empty_word = Word::new();
        let current_query = OutputQuery::new(empty_word);
        p.push(current_query.clone());

        let mut open_queries = VecDeque::new();
        open_queries.push_back(current_query);
        let mut close_queries = Vec::new();
        let mut seen_states = HashSet::new();
        seen_states.insert(hypothesis.initial_state.name.clone());

        while let Some(query) = open_queries.pop_front() {
            let mut tmp_seen_states = HashSet::new();

            for letter in &self.input_letters {
                let new_word = query
                    .input_word
                    .concatenate(&Word::from_letters(vec![letter.clone()]));
                let query_z = OutputQuery::new(new_word);

                if let Ok((_, visited_states)) = hypothesis.play_query(&query_z) {
                    close_queries.push(query_z.clone());

                    if let Some(last_state) = visited_states.last() {
                        if !seen_states.contains(&last_state.name) {
                            tmp_seen_states.insert(last_state.name.clone());
                            open_queries.push_back(query_z);
                        }
                    }
                }
            }

            seen_states.extend(tmp_seen_states);
        }

        p.extend(close_queries);
        p
    }

    fn compute_z(&self, hypothesis: &Automata, w: &[OutputQuery]) -> Vec<OutputQuery> {
        let mut z = Vec::new();
        let mut z_set = HashSet::new();
        
        for query in w {
            z_set.insert(query.input_word.clone());
            z.push(query.clone());
        }

        let states = hypothesis.get_states();
        let v = if self.max_states > states.len() {
            self.max_states - states.len()
        } else {
            0
        };

        let mut output_queries = Vec::new();
        for input_letter in &self.input_letters {
            output_queries.push(OutputQuery::new(Word::from_letters(vec![
                input_letter.clone()
            ])));
        }

        let mut x = vec![w.to_vec()];
        for i in 1..=v {
            let mut xi = Vec::new();
            let previous_x = &x[i - 1];

            for x_elem in previous_x {
                for oq in &output_queries {
                    let new_word = x_elem.input_word.concatenate(&oq.input_word);
                    xi.push(OutputQuery::new(new_word));
                }
            }

            for xi_elem in &xi {
                if z_set.insert(xi_elem.input_word.clone()) {
                    z.push(xi_elem.clone());
                }
            }

            x.push(xi);
        }

        z
    }
}

impl EquivalenceTest for WMethodEQ {
    fn find_counterexample(&self, hypothesis: &mut Automata) -> Option<Counterexample> {
        let states = hypothesis.get_states();
        let mut w = Vec::new();

        for i in 0..states.len() {
            for j in (i + 1)..states.len() {
                let couple = (&states[i], &states[j]);
                w.push(self.compute_distinguishable_string(hypothesis, couple));
            }
        }

        let p = self.compute_p(hypothesis);
        let z = self.compute_z(hypothesis, &w);
        let mut t = p;
        t.extend(z);

        for testcase_query in t.iter().skip(1) {
            if let Ok((hypothesis_output_word, _)) = hypothesis.play_query(testcase_query) {
                let mut query = testcase_query.clone();
                if self
                    .knowledge_base
                    .lock()
                    .unwrap()
                    .resolve_query(&mut query)
                    .is_ok()
                {
                    if let Some(real_output_word) = query.output_word() {
                        if real_output_word != &hypothesis_output_word {
                            return Some(Counterexample {
                                input_word: testcase_query.input_word.clone(),
                                output_word: real_output_word.clone(),
                            });
                        }
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
    fn wmethod_finds_counterexample_for_wrong_hypothesis() {
        let kb: Arc<Mutex<dyn KnowledgeBaseTrait>> =
            Arc::new(Mutex::new(ConstantOutputKb::new("1")));
        let eq = WMethodEQ::new(kb, vec![Letter::new("a")], 2);
        let mut hypothesis = build_single_state_hypothesis("0");

        let ce = eq.find_counterexample(&mut hypothesis);
        assert!(ce.is_some());
    }

    #[test]
    fn wmethod_returns_none_for_equivalent_hypothesis() {
        let kb: Arc<Mutex<dyn KnowledgeBaseTrait>> =
            Arc::new(Mutex::new(ConstantOutputKb::new("0")));
        let eq = WMethodEQ::new(kb, vec![Letter::new("a")], 2);
        let mut hypothesis = build_single_state_hypothesis("0");

        let ce = eq.find_counterexample(&mut hypothesis);
        assert!(ce.is_none());
    }
}
