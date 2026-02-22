/// Example: Using Random Walk Equivalence Test
use rust_lstar::eqtest::RandomWalkMethod;
use rust_lstar::knowledge_base::{KnowledgeBaseStats, KnowledgeBaseTrait};
use rust_lstar::query::OutputQuery;
use rust_lstar::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Random Walk Equivalence Test Example ===\n");

    // Create a simple system
    let mut kb = DemoKnowledgeBase::new();

    // Define a simple state machine:
    // S0 -a/0-> S1   (initial state)
    // S0 -b/1-> S0
    // S1 -a/0-> S2
    // S1 -b/1-> S0
    // S2 -a/0-> S2
    // S2 -b/1-> S1

    // Make outputs state-dependent so learner must distinguish states
    kb.add_transition("s0", "a", "0", "s1");
    kb.add_transition("s0", "b", "1", "s0");
    kb.add_transition("s1", "a", "1", "s2");
    kb.add_transition("s1", "b", "0", "s0");
    kb.add_transition("s2", "a", "1", "s2");
    kb.add_transition("s2", "b", "0", "s1");

    let knowledge_base = Arc::new(Mutex::new(kb));

    // Create vocabulary
    let vocabulary = vec!["a".to_string(), "b".to_string()];

    // Create learner
    let mut lstar = LSTAR::new(vocabulary.clone(), knowledge_base.clone(), 5, None, None);

    // Use random walk equivalence test instead of W-method
    let input_letters = vocabulary
        .iter()
        .map(|s| Letter::new(s))
        .collect::<Vec<_>>();
    let random_walk = RandomWalkMethod::new(
        knowledge_base.clone(),
        input_letters,
        10000, // max_steps: 10000 steps
        0.75,  // restart_probability: 75%
    );

    lstar = lstar.with_equivalence_test(Arc::new(random_walk));

    // Run learning
    match lstar.learn() {
        Ok(automata) => {
            println!("\n=== Learned Automaton (Random Walk Test) ===\n");
            println!("{}", automata.build_dot_code());
            println!("\nLearning completed successfully!");

            // Print statistics
            println!("\nStatistics:");
            println!("  Number of states: {}", automata.get_states().len());
            println!("  Number of transitions: {}", automata.transitions.len());
        }
        Err(e) => eprintln!("Error during learning: {}", e),
    }

    let kb_guard = knowledge_base.lock().unwrap();
    println!("\nKnowledge Base Statistics:\n{}", kb_guard.stats);

    Ok(())
}

struct DemoKnowledgeBase {
    transitions: HashMap<(String, String), (String, String)>,
    current_state: String,
    stats: KnowledgeBaseStats,
}

impl DemoKnowledgeBase {
    fn new() -> Self {
        Self {
            transitions: std::collections::HashMap::new(),
            current_state: "s0".to_string(),
            stats: KnowledgeBaseStats::new(),
        }
    }

    fn add_transition(&mut self, from_state: &str, input: &str, output: &str, to_state: &str) {
        self.transitions.insert(
            (from_state.to_string(), input.to_string()),
            (output.to_string(), to_state.to_string()),
        );
    }
}

impl KnowledgeBaseTrait for DemoKnowledgeBase {
    fn resolve_query(&mut self, query: &mut OutputQuery) -> Result<(), String> {
        self.stats.increment_nb_query();
        self.stats.add_nb_letter(query.input_word.len());
        self.stats.increment_nb_submitted_query();
        self.stats.add_nb_submitted_letter(query.input_word.len());

        self.current_state = "s0".to_string();
        let mut output_letters = Vec::new();

        for input_letter in query.input_word.letters() {
            let input = input_letter.symbols();
            let key = (self.current_state.clone(), input);
            let (output, next_state) = self
                .transitions
                .get(&key)
                .cloned()
                .ok_or_else(|| format!("No transition for ({}, {})", key.0, key.1))?;

            output_letters.push(Letter::new(output));
            self.current_state = next_state;
        }

        query.set_result(Word::from_letters(output_letters));
        Ok(())
    }

    fn add_word(&mut self, _input_word: &Word, _output_word: &Word) -> Result<(), String> {
        Ok(())
    }
}
