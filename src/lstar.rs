use crate::automata::Automata;
use crate::eqtest::{Counterexample, EquivalenceTest, WMethodEQ};
use crate::knowledge_base::KnowledgeBaseTrait;
use crate::letter::Letter;
use crate::observation_table::ObservationTable;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// The L* algorithm implementation
pub struct LSTAR {
    pub input_vocabulary: Vec<Letter>,
    pub knowledge_base: Arc<Mutex<dyn KnowledgeBaseTrait>>,
    pub tmp_dir: Option<String>,
    pub observation_table: Option<ObservationTable>,
    pub max_states: usize,
    pub equivalence_test: Arc<dyn EquivalenceTest>,
    stop_flag: bool,
}

impl LSTAR {
    /// Create a new LSTAR learner
    pub fn new(
        input_vocabulary: Vec<String>,
        knowledge_base: Arc<Mutex<dyn KnowledgeBaseTrait>>,
        max_states: usize,
        tmp_dir: Option<String>,
        eq_test: Option<Arc<dyn EquivalenceTest>>,
    ) -> Self {
        let input_letters = input_vocabulary
            .into_iter()
            .map(|s| Letter::new(s))
            .collect::<Vec<_>>();

        let observation_table =
            ObservationTable::new(input_letters.clone(), knowledge_base.clone());

        let equivalence_test = eq_test.unwrap_or_else(|| {
            Arc::new(WMethodEQ::new(
                knowledge_base.clone(),
                input_letters.clone(),
                max_states,
            ))
        });

        LSTAR {
            input_vocabulary: input_letters,
            knowledge_base,
            tmp_dir,
            observation_table: Some(observation_table),
            max_states,
            equivalence_test,
            stop_flag: false,
        }
    }

    /// Set a custom equivalence test
    pub fn with_equivalence_test(mut self, eq_test: Arc<dyn EquivalenceTest>) -> Self {
        self.equivalence_test = eq_test;
        self
    }

    /// Stop the learning process
    pub fn stop(&mut self) {
        self.stop_flag = true;
    }

    /// Run the L* algorithm
    pub fn learn(&mut self) -> Result<Automata, String> {
        let start_time = Instant::now();

        println!("Starting L* learning process");
        println!("Input alphabet size: {}", self.input_vocabulary.len());
        println!("Max states: {}", self.max_states);

        self.initialize()?;

        let mut round = 1;
        let mut hypothesis_valid = false;
        let mut hypothesis: Option<Automata> = None;

        while !hypothesis_valid && !self.stop_flag {
            println!("\n--- Round {} ---", round);

            // Build hypothesis
            hypothesis = Some(self.build_hypothesis(round)?);
            println!("Hypothesis built");

            let _ = self.serialize_hypothesis(round, hypothesis.as_ref().unwrap());

            let counter_example = self
                .equivalence_test
                .find_counterexample(hypothesis.as_mut().unwrap());
            if let Some(ce) = counter_example {
                println!("Counterexample found: {:?}", ce);
                self.fix_hypothesis(ce)?;
            } else {
                println!("No counterexample found, hypothesis is correct!");
                hypothesis_valid = true;
            }

            round += 1;
        }

        self.serialize_observation_table(round)?;
        let duration = start_time.elapsed();
        println!("\nLearning completed in {:.2?}", duration);

        if let Some(hyp) = hypothesis {
            Ok(hyp)
        } else {
            Err("Failed to build a valid hypothesis".into())
        }
    }

    fn serialize_hypothesis(&self, round: usize, hypothesis: &Automata) -> Result<(), String> {
        let dot_code = hypothesis.build_dot_code();
        let filepath = if let Some(tmp_dir) = &self.tmp_dir {
            format!("{}/hypothesis_round_{}.dot", tmp_dir, round)
        } else {
            format!("hypothesis_round_{}.dot", round)
        };

        if let Some(parent) = std::path::Path::new(&filepath).parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }
        std::fs::write(&filepath, dot_code)
            .map_err(|e| format!("Failed to write hypothesis DOT file: {}", e))?;
        println!("Hypothesis for round {} serialized to {}", round, filepath);
        Ok(())
    }

    fn serialize_observation_table(&self, round: usize) -> Result<(), String> {
        let serialized_table = if let Some(ot) = &self.observation_table {
            ot.serialize()
        } else {
            return Err("No observation table available".into());
        };
        let str_date = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
        let filepath = if let Some(tmp_dir) = &self.tmp_dir {
            format!(
                "{}/observation_table_round_{}_{}.raw",
                tmp_dir, round, str_date
            )
        } else {
            format!("tmp/observation_table_round_{}_{}.raw", round, str_date)
        };

        if let Some(parent) = std::path::Path::new(&filepath).parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }
        std::fs::write(&filepath, serialized_table)
            .map_err(|e| format!("Failed to write observation table file: {}", e))?;
        println!(
            "Observation table for round {} serialized to {}",
            round, filepath
        );
        Ok(())
    }

    fn fix_hypothesis(&mut self, counter_example: Counterexample) -> Result<(), String> {
        println!(
            "Refining observation table with counterexample: {:?}",
            counter_example
        );

        let input_word = counter_example.input_word;
        let output_word = counter_example.output_word;
        if let Some(ot) = &mut self.observation_table {
            ot.add_counterexample(&input_word, &output_word)?;
        } else {
            return Err("No observation table available".into());
        }
        Ok(())
    }

    /// Build a hypothesis from the current observation table
    fn build_hypothesis(&mut self, round: usize) -> Result<Automata, String> {
        let mut f_consistent = false;
        let mut f_closed = false;
        while !f_consistent || !f_closed {
            if let Some(ot) = &mut self.observation_table {
                // Make table closed
                if !ot.is_closed() {
                    println!("  Closing table...");
                    ot.close_table()?;

                    f_closed = false;
                } else {
                    println!("  Table is closed");
                    f_closed = true;
                }

                // Check consistency
                if let Some(inconsistency) = ot.find_inconsistency() {
                    // println!("  Making table consistent...");
                    ot.make_consistent(inconsistency)?;
                    f_consistent = false;
                } else {
                    f_consistent = true;
                }
            } else {
                return Err("No observation table available".into());
            }
        }
        self.serialize_observation_table(round)?;

        if let Some(ot) = &mut self.observation_table {
            ot.build_hypothesis()
        } else {
            Err("No observation table available".into())
        }
    }

    fn initialize(&mut self) -> Result<(), String> {
        if let Some(ot) = &mut self.observation_table {
            ot.initialize()?;
            Ok(())
        } else {
            Err("Failed to initialize observation table".into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::knowledge_base::KnowledgeBase;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_lstar_creation() {
        let kb: Arc<Mutex<dyn crate::knowledge_base::KnowledgeBaseTrait>> =
            Arc::new(Mutex::new(KnowledgeBase::new()));
        let vocabulary = vec!["a".to_string(), "b".to_string()];
        let lstar = LSTAR::new(vocabulary, kb, 5, None, None);
        assert_eq!(lstar.input_vocabulary.len(), 2);
    }
}
