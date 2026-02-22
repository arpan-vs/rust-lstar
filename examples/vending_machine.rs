/// Example: Simple Vending Machine Learning
use rust_lstar::knowledge_base::{KnowledgeBaseStats, KnowledgeBaseTrait};
use rust_lstar::query::OutputQuery;
use rust_lstar::*;
use std::sync::{Arc, Mutex};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Vending Machine Learning Example ===\n");

    // Create a vending machine knowledge base (SUL)
    let kb = Arc::new(Mutex::new(VendingMachineKB::new()));

    // Define input vocabulary
    let vocabulary = vec![
        "INSERT_COIN".to_string(),
        "PRESS_A".to_string(),
        "PRESS_B".to_string(),
    ];

    // Run L* learner
    let mut learner = LSTAR::new(vocabulary, kb.clone(), 4, None, None);
    match learner.learn() {
        Ok(automata) => {
            println!("\n=== Learned Vending Machine ===\n");
            println!("{}", automata.build_dot_code());
        }
        Err(e) => eprintln!("Learning error: {}", e),
    }

    let kb_guard = kb.lock().unwrap();
    println!("\nKnowledge Base Statistics:\n{}", kb_guard.stats);

    Ok(())
}

/// Knowledge base that simulates the vending machine SUL
struct VendingMachineKB {
    state: VendingState,
    stats: KnowledgeBaseStats,
}

#[derive(Clone, Copy)]
enum VendingState {
    Idle,
    HasCoin,
}

impl VendingMachineKB {
    fn new() -> Self {
        VendingMachineKB {
            state: VendingState::Idle,
            stats: KnowledgeBaseStats::new(),
        }
    }

    fn process_input(&self, input: &str, current: VendingState) -> (VendingState, &'static str) {
        match current {
            VendingState::Idle => match input {
                "INSERT_COIN" => (VendingState::HasCoin, "BEEP"),
                "PRESS_A" => (VendingState::Idle, "ERROR"),
                "PRESS_B" => (VendingState::Idle, "ERROR"),
                _ => (VendingState::Idle, "ERROR"),
            },
            VendingState::HasCoin => match input {
                "PRESS_A" => (VendingState::Idle, "DISPENSE_A"),
                "PRESS_B" => (VendingState::Idle, "DISPENSE_B"),
                "INSERT_COIN" => (VendingState::HasCoin, "BEEP"),
                _ => (VendingState::HasCoin, "ERROR"),
            },
        }
    }
}

impl KnowledgeBaseTrait for VendingMachineKB {
    fn resolve_query(&mut self, query: &mut OutputQuery) -> Result<(), String> {
        self.stats.increment_nb_query();
        self.stats.add_nb_letter(query.input_word.len());
        self.stats.increment_nb_submitted_query();
        self.stats.add_nb_submitted_letter(query.input_word.len());

        // Reset to initial state for each query
        self.state = VendingState::Idle;

        let mut outputs = Vec::new();
        for input_letter in query.input_word.letters() {
            let command_string = input_letter.symbols();
            let command = command_string.as_str();

            let (next, resp) = self.process_input(command, self.state);
            self.state = next;
            outputs.push(Letter::new(resp));
        }

        query.set_result(Word::from_letters(outputs));
        Ok(())
    }

    fn add_word(&mut self, _input: &Word, _output: &Word) -> Result<(), String> {
        Ok(())
    }
}
