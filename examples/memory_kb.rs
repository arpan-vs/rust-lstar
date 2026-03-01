//! Demonstrates learning a randomly generated automaton using a custom
//! in-memory knowledge base layered on top of the default cache.

use rust_lstar::knowledge_base::{KnowledgeBase, KnowledgeBaseStats, KnowledgeBaseTrait};
use rust_lstar::query::OutputQuery;
use rust_lstar::*;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Memory Knowledge Base Learning Example ===\n");

    let input_symbols = vec!["a", "b", "c"];
    let output_symbols = vec!["0", "1", "2", "3"];
    let target_state_count = 5;
    let max_states = 8;

    let target = build_random_machine(target_state_count, &input_symbols, &output_symbols);
    println!(
        "Random target generated (states={}, transitions={})",
        target.get_states().len(),
        target.transitions.len()
    );

    let kb = Arc::new(Mutex::new(RandomMachineKnowledgeBase::new(target.clone())));
    let vocabulary = input_symbols
        .iter()
        .map(|symbol| symbol.to_string())
        .collect::<Vec<_>>();

    let mut learner = LSTAR::new(vocabulary, kb.clone(), max_states, None, None);
    let learned = learner.learn()?;

    println!("\n=== Learned Automaton ===");
    println!("{}", learned.build_dot_code());

    let kb_guard = kb.lock().unwrap();
    println!("\nKnowledge Base Statistics:\n{}", kb_guard.stats());

    Ok(())
}

/// Knowledge base that answers queries by executing a hidden target automaton.
struct RandomMachineKnowledgeBase {
    base: KnowledgeBase,
    target: Automata,
}

impl RandomMachineKnowledgeBase {
    fn new(target: Automata) -> Self {
        Self {
            base: KnowledgeBase::new(),
            target,
        }
    }

    fn stats(&self) -> &KnowledgeBaseStats {
        self.base.stats()
    }

    fn submit_word_to_target(&mut self, word: &Word) -> Result<Word, String> {
        if word.is_empty() {
            return Ok(Word::new());
        }

        let initial_state = self.target.initial_state.clone();
        let (output_word, _) = self
            .target
            .play_word(word, Some(&initial_state))?;
        Ok(output_word)
    }
}

impl KnowledgeBaseTrait for RandomMachineKnowledgeBase {
    fn resolve_query(&mut self, query: &mut OutputQuery) -> Result<(), String> {
        match self.base.resolve_query(query) {
            Ok(_) => Ok(()),
            Err(_) => {
                let output = self.submit_word_to_target(&query.input_word)?;
                self.base.add_word(&query.input_word, &output)?;
                query.set_result(output);
                Ok(())
            }
        }
    }

    fn add_word(&mut self, input_word: &Word, output_word: &Word) -> Result<(), String> {
        self.base.add_word(input_word, output_word)
    }
}

fn build_random_machine(
    state_count: usize,
    input_symbols: &[&str],
    output_symbols: &[&str],
) -> Automata {
    let mut rng = SimpleRng::new();
    let mut automata = Automata::new(State::new("S0".to_string()), "RandomTarget".to_string());
    let mut transitions = Vec::new();
    let mut transition_id = 0usize;

    for state_idx in 0..state_count {
        for input_symbol in input_symbols {
            let next_state_idx = rng.gen_range(0, state_count);
            let output_idx = rng.gen_range(0, output_symbols.len());

            transitions.push(Transition::new(
                format!("t{}", transition_id),
                State::new(format!("S{}", state_idx)),
                State::new(format!("S{}", next_state_idx)),
                Letter::new(*input_symbol),
                Letter::new(output_symbols[output_idx]),
            ));
            transition_id += 1;
        }
    }

    automata.transitions = transitions;
    automata
}

/// Tiny deterministic RNG for generating repeatable random automata topology.
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new() -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(1664525).wrapping_add(1013904223);
        self.state
    }

    fn gen_range(&mut self, min: usize, max: usize) -> usize {
        if max <= min {
            return min;
        }
        min + (self.next_u64() as usize) % (max - min)
    }
}
