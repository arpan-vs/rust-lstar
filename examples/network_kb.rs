/// Example: Network Active Knowledge Base
/// This example shows how to use the NetworkActiveKnowledgeBase to communicate
/// with a remote target system via network sockets.
use rust_lstar::knowledge_base::{KnowledgeBaseTrait, NetworkActiveKnowledgeBase};
use rust_lstar::*;
use std::sync::{Arc, Mutex};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Network Active Knowledge Base Example ===\n");

    // This example expects an independent coffee server running
    // (see examples/coffee_server.rs).
    let kb = NetworkActiveKnowledgeBase::new("127.0.0.1".to_string(), 3000, Duration::from_secs(5));

    // Define input vocabulary
    let vocabulary = vec![
        "REFILL_WATER".to_string(),
        "REFILL_COFFEE".to_string(),
        "PRESS_BUTTON_A".to_string(),
        "PRESS_BUTTON_B".to_string(),
        "PRESS_BUTTON_C".to_string(),
    ];

    println!("Target Host: {}", kb.target_host());
    println!("Target Port: {}\n", kb.target_port());

    let kb: Arc<Mutex<dyn KnowledgeBaseTrait>> = Arc::new(Mutex::new(kb));

    // Create learner. The network KB connects per submitted word.
    let mut learner = LSTAR::new(vocabulary, kb, 4, None, None);

    match learner.learn() {
        Ok(automata) => {
            println!("\n=== Learned Automaton from Network Target ===\n");
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
            println!("  1. Start a coffee server: cargo run --example coffee_server");
            println!("  2. Run this example again");
        }
    }

    Ok(())
}
