/// Benchmark example to demonstrate optimization improvements
use rust_lstar::knowledge_base::{KnowledgeBaseStats, KnowledgeBaseTrait};
use rust_lstar::query::OutputQuery;
use rust_lstar::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Performance Benchmark ===\n");

    // Create a more complex system to test performance
    let mut kb = BenchmarkKnowledgeBase::new();

    // Define a 4-state machine with 3 input symbols
    // This creates a more complex learning scenario
    kb.add_transition("s0", "a", "0", "s1");
    kb.add_transition("s0", "b", "1", "s2");
    kb.add_transition("s0", "c", "0", "s0");
    
    kb.add_transition("s1", "a", "1", "s2");
    kb.add_transition("s1", "b", "0", "s3");
    kb.add_transition("s1", "c", "1", "s1");
    
    kb.add_transition("s2", "a", "0", "s3");
    kb.add_transition("s2", "b", "1", "s0");
    kb.add_transition("s2", "c", "0", "s2");
    
    kb.add_transition("s3", "a", "1", "s0");
    kb.add_transition("s3", "b", "0", "s1");
    kb.add_transition("s3", "c", "1", "s3");

    let knowledge_base = Arc::new(Mutex::new(kb));
    let vocabulary = vec!["a".to_string(), "b".to_string(), "c".to_string()];

    println!("Learning 4-state automaton with 3-symbol alphabet...\n");
    
    let start = Instant::now();
    let mut lstar = LSTAR::new(vocabulary, knowledge_base.clone(), 6, None, None);
    let automata = lstar.learn()?;
    let duration = start.elapsed();

    println!("\n=== Results ===" );
    println!("Learning time: {:.2?}", duration);
    println!("States learned: {}", automata.get_states().len());
    println!("Transitions: {}", automata.transitions.len());
    
    let kb_guard = knowledge_base.lock().unwrap();
    println!("\nQuery Statistics:");
    println!("  Total queries: {}", kb_guard.stats.nb_query());
    println!("  Total letters processed: {}", kb_guard.stats.nb_letter());
    println!("  Avg letters per query: {:.2}", 
        kb_guard.stats.nb_letter() as f64 / kb_guard.stats.nb_query() as f64);

    println!("\n=== Optimization Benefits ===");
    println!("✓ Cached transition index: Fast automata playback");
    println!("✓ SmallVec: Reduced allocations for short words");
    println!("✓ Enum Letter: Optimized single-symbol storage");
    println!("✓ HashSet dedup: O(1) query deduplication");
    println!("✓ Pre-allocated collections: Fewer reallocations");

    Ok(())
}

struct BenchmarkKnowledgeBase {
    transitions: HashMap<(String, String), (String, String)>,
    current_state: String,
    stats: KnowledgeBaseStats,
}

impl BenchmarkKnowledgeBase {
    fn new() -> Self {
        Self {
            transitions: HashMap::new(),
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

impl KnowledgeBaseTrait for BenchmarkKnowledgeBase {
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
