//! Learn the ATM protocol over TCP via `NetworkActiveKnowledgeBase`.
//!
//! Run with:
//! 1. `cargo run --example custom_kb_server`
//! 2. `cargo run --example network_custom_kb`

use rust_lstar::knowledge_base::{KnowledgeBaseTrait, NetworkActiveKnowledgeBase};
use rust_lstar::*;
use std::sync::{Arc, Mutex};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Network Custom KB (ATM) Example ===\n");

    // Expects an independent ATM server running
    // (see examples/custom_kb_server.rs).
    let kb = NetworkActiveKnowledgeBase::new("127.0.0.1".to_string(), 3001, Duration::from_secs(5));

    let vocabulary = vec![
        "INSERT_CARD".to_string(),
        "ENTER_PIN".to_string(),
        "REQUEST_WITHDRAW".to_string(),
        "EJECT_CARD".to_string(),
        "TIMEOUT".to_string(),
    ];

    println!("Target Host: {}", kb.target_host());
    println!("Target Port: {}\n", kb.target_port());

    let kb: Arc<Mutex<dyn KnowledgeBaseTrait>> = Arc::new(Mutex::new(kb));
    let mut learner = LSTAR::new(vocabulary, kb, 8, None, None);

    match learner.learn() {
        Ok(automata) => {
            println!("\n=== Learned ATM Automaton from Network Target ===\n");
            println!("{}", automata.build_dot_code());
            println!(
                "\nStates: {}\nTransitions: {}",
                automata.get_states().len(),
                automata.transitions.len()
            );
        }
        Err(e) => {
            eprintln!("Learning error: {}", e);
            println!("\nTo run this example with a real connection:");
            println!("  1. Start ATM server: cargo run --example custom_kb_server");
            println!("  2. Run this example again: cargo run --example network_custom_kb");
        }
    }

    Ok(())
}
