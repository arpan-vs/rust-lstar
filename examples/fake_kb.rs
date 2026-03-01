/// Example: Using FakeActiveKnowledgeBase with preset automaton
use rust_lstar::automata::{Automata, State, Transition};
use rust_lstar::knowledge_base::FakeActiveKnowledgeBase;
use rust_lstar::*;
use std::sync::{Arc, Mutex};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== FakeActiveKnowledgeBase Example ===\n");

    // Build a simple automaton to use as target
    let s0 = State::new("s0".to_string());
    let s1 = State::new("s1".to_string());

    let mut target = Automata::new(s0.clone(), "Target".to_string());

    // Add all transitions to the flat transitions list
    target.transitions.push(Transition::new(
        "t0".to_string(),
        s0.clone(),
        s1.clone(),
        Letter::new("a"),
        Letter::new("0"),
    ));
    target.transitions.push(Transition::new(
        "t1".to_string(),
        s0.clone(),
        s0.clone(),
        Letter::new("b"),
        Letter::new("1"),
    ));
    target.transitions.push(Transition::new(
        "t2".to_string(),
        s1.clone(),
        s1.clone(),
        Letter::new("a"),
        Letter::new("1"),
    ));
    target.transitions.push(Transition::new(
        "t3".to_string(),
        s1.clone(),
        s0.clone(),
        Letter::new("b"),
        Letter::new("0"),
    ));

    println!("Automata:\n{}", target.build_dot_code());
    let kb = FakeActiveKnowledgeBase::new(target);
    let knowledge_base = Arc::new(Mutex::new(kb));

    let vocabulary = vec!["a".to_string(), "b".to_string()];
    let mut lstar = LSTAR::new(vocabulary, knowledge_base, 5, None, None);

    let automata = lstar.learn()?;
    println!("\n=== Learned Automaton ===\n");
    println!("{}", automata.build_dot_code());

    Ok(())
}
