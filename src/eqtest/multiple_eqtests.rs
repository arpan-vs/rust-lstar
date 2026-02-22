use crate::automata::Automata;
use crate::eqtest::{Counterexample, EquivalenceTest};
use std::sync::Arc;

/// Compose multiple equivalence strategies and return the first counterexample.
pub struct MultipleEqtests {
    pub eqtests: Vec<Arc<dyn EquivalenceTest>>,
}

impl MultipleEqtests {
    pub fn new(eqtests: Vec<Arc<dyn EquivalenceTest>>) -> Self {
        Self { eqtests }
    }
}

impl EquivalenceTest for MultipleEqtests {
    fn find_counterexample(&self, hypothesis: &mut Automata) -> Option<Counterexample> {
        for eqtest in &self.eqtests {
            if let Some(counterexample) = eqtest.find_counterexample(hypothesis) {
                return Some(counterexample);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::automata::{Automata, State};
    use crate::eqtest::EquivalenceTest;
    use crate::letter::Letter;
    use crate::word::Word;

    struct AlwaysNone;
    impl EquivalenceTest for AlwaysNone {
        fn find_counterexample(&self, _hypothesis: &mut Automata) -> Option<Counterexample> {
            None
        }
    }

    struct AlwaysSome;
    impl EquivalenceTest for AlwaysSome {
        fn find_counterexample(&self, _hypothesis: &mut Automata) -> Option<Counterexample> {
            Some(Counterexample {
                input_word: Word::from_letters(vec![Letter::new("a")]),
                output_word: Word::from_letters(vec![Letter::new("1")]),
            })
        }
    }

    #[test]
    fn multiple_eqtests_returns_none_when_all_none() {
        let mut hypothesis = Automata::new(State::new("S0".to_string()), "H".to_string());
        let eq = MultipleEqtests::new(vec![Arc::new(AlwaysNone), Arc::new(AlwaysNone)]);
        assert!(eq.find_counterexample(&mut hypothesis).is_none());
    }

    #[test]
    fn multiple_eqtests_returns_first_counterexample() {
        let mut hypothesis = Automata::new(State::new("S0".to_string()), "H".to_string());
        let eq = MultipleEqtests::new(vec![
            Arc::new(AlwaysNone),
            Arc::new(AlwaysSome),
            Arc::new(AlwaysNone),
        ]);
        assert!(eq.find_counterexample(&mut hypothesis).is_some());
    }
}
