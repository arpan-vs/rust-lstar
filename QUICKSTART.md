# Quick Start Guide - rust-lstar

Get started with the L* learning algorithm in 5 minutes!

## Installation

You need Rust. If you don't have it, install from [https://rustup.rs/](https://rustup.rs/)

```bash
# Clone or download the repository
cd rust-lstar

# Build the project
cargo build --release

# Run the demo
cargo run
```

## Basic Usage

### 1. Create a Knowledge Base

Implement the `KnowledgeBase` trait to define your system:

```rust
use rust_lstar::{KnowledgeBase, OutputQuery, Word, Letter};
use std::sync::Arc;

struct MySystem;

impl KnowledgeBase for MySystem {
    fn resolve_query(&self, query: &mut OutputQuery) -> Result<(), String> {
        // For each input, produce an output
        let outputs: Vec<Letter> = query.input_word
            .letters()
            .iter()
            .map(|_| Letter::new("output"))
            .collect();
        
        query.set_result(Word::from_letters(outputs));
        Ok(())
    }
}
```

### 2. Define Your Alphabet

```rust
let vocabulary = vec![
    "a".to_string(),
    "b".to_string(),
    "c".to_string(),
];
```

### 3. Run the Learner

```rust
use rust_lstar::LSTAR;
use std::sync::Arc;

let kb = Arc::new(MySystem);
let mut lstar = LSTAR::new(vocabulary, kb, 10); // max 10 states

match lstar.learn() {
    Ok(automata) => {
        println!("{}", automata.build_dot_code());
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

## Running Examples

Three examples are provided:

```bash
# Simple vending machine simulation
cargo run --example vending_machine

# Using random walk equivalence testing
cargo run --example random_walk_eq

# Custom knowledge base (ATM protocol)
cargo run --example custom_kb
```

## Complete Minimal Example

```rust
use rust_lstar::*;
use std::sync::Arc;

// Simple knowledge base that echoes 'X' for each input
struct EchoKB;

impl KnowledgeBase for EchoKB {
    fn resolve_query(&self, query: &mut OutputQuery) -> Result<(), String> {
        let outputs = query.input_word
            .letters()
            .iter()
            .map(|_| Letter::new("X"))
            .collect::<Vec<_>>();
        query.set_result(Word::from_letters(outputs));
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let kb = Arc::new(EchoKB);
    let vocab = vec!["a".to_string(), "b".to_string()];
    
    let mut lstar = LSTAR::new(vocab, kb, 5);
    let automata = lstar.learn()?;
    
    println!("{}", automata.build_dot_code());
    Ok(())
}
```

## Key Concepts

### Knowledge Base
Represents your "System Under Learning" - something you want to model

### Vocabulary
The alphabet of input commands your system understands

### Automata
The learned state machine - outputs in Graphviz DOT format

### Equivalence Test
Strategy for checking if the learned automaton is correct:
- **W-method**: Exhaustive (default)
- **RandomWalk**: Probabilistic (faster)

## Customizing Equivalence Testing

Use a different equivalence test:

```rust
use rust_lstar::equivalence_test::RandomWalkMethod;

let input_letters = vocab.iter().map(|s| Letter::new(s)).collect();
let random_walk = RandomWalkMethod::new(
    kb.clone(), 
    input_letters,
    100,    // num_tries
    2.0,    // walk_length_ratio
);

let mut lstar = LSTAR::new(vocab, kb, 5)
    .with_equivalence_test(Arc::new(random_walk));
```

## Testing

Run all tests:
```bash
cargo test
```

Run only library tests:
```bash
cargo test --lib
```

Run a specific example:
```bash
cargo test --example vending_machine
```

## Output

The learned automaton is printed in Graphviz DOT format:

```dot
digraph "Automata" {
    "0" [shape=doubleoctagon, ...];
    "1" [shape=ellipse, ...];
    "0" -> "1" [label="a / output", ...];
}
```

### Visualize the automaton:
```bash
# Save to graphviz
cargo run > automata.dot

# Convert to image (requires graphviz)
dot -Tpng automata.dot -o automata.png
```

## Debugging

Enable detailed logging:
```bash
RUST_LOG=info cargo run
```

## Common Issues

### "No initial state found"
The observation table couldn't build a valid automaton. Usually means:
- System is not deterministic
- Vocabulary is incomplete
- Max states too small

### "Query execution failed"
The knowledge base returned an error:
- Check your `resolve_query()` implementation
- Ensure it produces output for all inputs

### Learning takes too long
Try:
- Reducing max_states
- Using RandomWalkMethod instead of W-method
- Checking if system is actually learnable

## Next Steps

1. **Read the README**: `README.md` for full documentation
2. **Explore Examples**: Check `examples/` directory
3. **Review Design**: `DESIGN.md` for architecture details
4. **Implement Your System**: Create custom knowledge base

## Tips & Tricks

### Performance (v0.2.0+)
```rust
// Optimizations are automatic:
// - Cached transition lookups (30-50% faster)
// - SmallVec for short words (no heap allocation)
// - Reduced cloning (20-30% fewer allocations)
// Overall: 2-3x faster for large automata
```

### Faster Learning
```rust
// Use random walk instead of W-method
let random_walk = RandomWalkMethod::new(kb, letters, 50, 1.5);
let lstar = LSTAR::new(vocab, kb, states)
    .with_equivalence_test(Arc::new(random_walk));
```

### Better Understanding
```rust
// Enable logging
env_logger::init();
RUST_LOG=info cargo run
```

### Testing Your System
```rust
// Use FakeKnowledgeBase until you're ready
let kb = Arc::new(FakeKnowledgeBase::new());
```

### Exporting Results
```rust
// Print automaton
println!("{}", automata.build_dot_code());

// Access internal structure
for state in &automata.states {
    println!("State: {}", state.name);
    for transition in &state.transitions {
        println!("  {} -> {}", transition.input_letter, transition.output_letter);
    }
}
```

## Resources

- [L* Algorithm Paper](https://dl.acm.org/doi/10.1016/0890-5401(87)90052-6)
- [Graphviz Visualization](https://graphviz.org/)
- [Original pylstar](https://github.com/gbossert/pylstar)

## Getting Help

- Check examples in `examples/` directory
- Read `README.md` for detailed documentation
- Review `DESIGN.md` for architecture understanding
- Look at unit tests in `src/` for usage patterns

Happy learning! 🚀
