use rust_lstar::knowledge_base::{KnowledgeBaseStats, KnowledgeBaseTrait};
use rust_lstar::query::OutputQuery;
/// Example: Custom Knowledge Base Implementation
/// This example shows how to implement your own knowledge base
/// for a real system (in practice, this could be a network service, hardware device, etc.)
use rust_lstar::*;
use std::sync::{Arc, Mutex};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Custom Knowledge Base Example ===");
    println!("Simulating a banking ATM protocol\n");

    let kb = Arc::new(Mutex::new(ATMKnowledgeBase::new()));

    let vocabulary = vec![
        "INSERT_CARD".to_string(),
        "ENTER_PIN".to_string(),
        "REQUEST_WITHDRAW".to_string(),
        "EJECT_CARD".to_string(),
        "TIMEOUT".to_string(),
    ];

    let mut lstar = LSTAR::new(vocabulary, kb.clone(), 8, None, None);

    match lstar.learn() {
        Ok(automata) => {
            println!("\n=== Learned ATM Protocol ===");
            println!("{}", automata.build_dot_code());
            println!("\nATM protocol learned successfully!");
        }
        Err(e) => eprintln!("Error: {}", e),
    }

    let kb_guard = kb.lock().unwrap();
    println!("\nKnowledge Base Stats:\n{}", kb_guard.stats);

    Ok(())
}

/// Custom knowledge base for an ATM system
struct ATMKnowledgeBase {
    state: ATMState,
    stats: KnowledgeBaseStats,
}

#[derive(Clone, Copy)]
enum ATMState {
    Idle,
    CardInserted,
    Authenticated,
    Ready,
    Dispensing,
}

impl ATMKnowledgeBase {
    fn new() -> Self {
        ATMKnowledgeBase {
            state: ATMState::Idle,
            stats: KnowledgeBaseStats::new(),
        }
    }

    fn process_input(&self, command: &str, current_state: ATMState) -> (ATMState, &'static str) {
        match current_state {
            ATMState::Idle => match command {
                "INSERT_CARD" => (ATMState::CardInserted, "CARD_ACCEPTED"), // CARD_ACCEPTED
                _ => (ATMState::Idle, "INVALID_OP"),                        // INVALID_OP
            },
            ATMState::CardInserted => match command {
                "ENTER_PIN" => (ATMState::Authenticated, "PIN_VERIFIED"), // PIN_VERIFIED
                "EJECT_CARD" => (ATMState::Idle, "CARD_EJECTED"),         // CARD_EJECTED
                _ => (ATMState::CardInserted, "RETRY"),                   // RETRY
            },
            ATMState::Authenticated => match command {
                "REQUEST_WITHDRAW" => (ATMState::Ready, "ENTER_AMOUNT"), // ENTER_AMOUNT
                "EJECT_CARD" => (ATMState::Idle, "CARD_EJECTED"),        // CARD_EJECTED
                "TIMEOUT" => (ATMState::Idle, "SESSION_TIMEOUT"),        // SESSION_TIMEOUT
                _ => (ATMState::Authenticated, "INVALID_COMMAND"),       // INVALID
            },
            ATMState::Ready => match command {
                "REQUEST_WITHDRAW" => (ATMState::Dispensing, "DISPENSING"), // DISPENSING
                "EJECT_CARD" => (ATMState::Idle, "CARD_EJECTED"),           // CARD_EJECTED
                _ => (ATMState::Ready, "WAIT"),                             // WAIT
            },
            ATMState::Dispensing => match command {
                "EJECT_CARD" => (ATMState::Idle, "CARD_EJECTED"), // CARD_EJECTED
                _ => (ATMState::Dispensing, "DISPENSING"),        // DISPENSING
            },
        }
    }
}

impl KnowledgeBaseTrait for ATMKnowledgeBase {
    fn resolve_query(&mut self, query: &mut OutputQuery) -> Result<(), String> {
        self.stats.increment_nb_query();
        self.stats.add_nb_letter(query.input_word.len());
        self.stats.increment_nb_submitted_query();
        self.stats.add_nb_submitted_letter(query.input_word.len());
        // Reset to initial state for each query
        self.state = ATMState::Idle;

        // let mut state = self.state.lock().unwrap();
        let mut outputs = Vec::new();

        for input_letter in query.input_word.letters() {
            // Extract command from letter name (format: "Letter('x')")
            let symbol = input_letter.symbols();
            let command = symbol.as_str();

            let (next_state, response) = self.process_input(command, self.state);
            self.state = next_state;
            outputs.push(Letter::new(response));
        }

        query.set_result(Word::from_letters(outputs));
        Ok(())
    }

    fn add_word(&mut self, _input: &Word, _output: &Word) -> Result<(), String> {
        // Not needed for this simple example
        Ok(())
    }

    // fn stats(&self) -> &KnowledgeBaseStats {
    //     &self.stats
    // }

    // fn stats_mut(&mut self) -> &mut KnowledgeBaseStats {
    //     &mut self.stats
    // }

    // fn load_cache(&mut self, _cache_path: &PathBuf) -> Result<(), String> {
    //     Ok(())
    // }

    // fn write_cache(&self, _cache_path: &PathBuf) -> Result<(), String> {
    //     Ok(())
    // }

    // fn clear_cache(&mut self) {
    //     // Not needed for this simple example
    // }
}
