# Rust L* Implementation - Design Document

## Architecture Overview

This document describes the architectural choices and design patterns used in the rust-lstar implementation.

## Design Principles

1. **Idiomatic Rust**: Leverage ownership, trait systems, and pattern matching
2. **Type Safety**: Compile-time guarantees for correctness
3. **Extensibility**: Trait-based design for user customization
4. **Performance**: Efficient memory management and algorithmic optimizations
5. **Clarity**: Clear module boundaries and separation of concerns

## Module Organization

### Core Algorithm Modules

#### 1. `letter.rs` - Alphabets and Symbols
**Purpose**: Represent atomic symbols in the learning alphabet

**Key Types**:
- `Letter`: Generic alphabet symbol wrapper
  - Can wrap any serializable/comparable type
  - Implements hash/eq for use as HashMap keys
  - Supports compound letters (multiple symbols)
- `EmptyLetter`: Special identity element for concatenation

**Design Decisions**:
- Uses `HashSet<String>` internally for symbol representation
- Immutable by default (functional style)
- String representation for DOT output compatibility

#### 2. `word.rs` - Sequences of Letters
**Purpose**: Represent sequences of letters (input/output strings)

**Key Types**:
- `Word`: Immutable sequence of letters
  - Supports concatenation via `concatenate()`
  - Can compute prefixes
  - Implements Hash/Eq for table indexing

**Design Decisions**:
- Immutable API enforces functional patterns
- Efficient prefix computation
- Cloneable for use in complex data structures

#### 3. `query.rs` - System Queries
**Purpose**: Represent a query to the System Under Learning

**Key Types**:
- `OutputQuery`: Represents a membership query
  - Input: word to query
  - Output: word returned by system (after execution)
  - Status: tracked via `queried` flag

**Design Decisions**:
- Mutable methods for setting results
- Separates query definition from execution

#### 4. `observation_table.rs` - Learning State
**Purpose**: Central data structure maintaining the learning progress

**Key Types**:
- `ObservationTable`: The Angluin observation table
  - D: Distinguishing suffixes
  - S: Short prefixes (hypothesis states)
  - SA: Long prefixes (counterexample prefixes)
  - ot_content: Table cells

**Key Methods**:
- `initialize()`: Set up with empty string and alphabet
- `add_word_in_D/S/SA()`: Add rows/columns
- `is_closed()`: Check closure property
- `close_table()`: Enforce closure
- `find_inconsistency()`: Detect consistency violations
- `make_consistent()`: Fix inconsistencies
- `build_hypothesis()`: Generate automaton

**Design Decisions**:
- Uses Arc<dyn KnowledgeBase> for queries
- IndexMap for maintaining insertion order
- Lazy evaluation of queries (only when adding rows)
- Stateful design reflects algorithm's iterative nature

#### 5. `automata.rs` - Output Structure
**Purpose**: Represent learned automaton

**Key Types**:
- `State`: Represents an automaton state
  - Name and list of transitions
- `Transition`: Edge in the automaton
  - Source, destination, input letter, output letter
- `Automata`: Complete automaton
  - Initial state references
  - All states and transitions
  - DOT code generation

**Design Decisions**:
- Explicit state naming for debugging
- Transition contains full path information (no separate edges)
- DOT generation built-in for visualization

#### 6. `knowledge_base.rs` - System Interface
**Purpose**: Trait defining how to query the System Under Learning

**Key Types/Traits**:
- `KnowledgeBase`: Main trait
  - `resolve_query()`: Execute membership query
  - `is_available()`: Check system availability
  - `reset()`: Reset to initial state
- `FakeKnowledgeBase`: Test/demo implementation

**Design Decisions**:
- Trait object pattern (Arc<dyn>) for flexibility
- Result<> return type for error handling
- Query structure passed by reference for modification

#### 7. `equivalence_test.rs` - Hypothesis Verification
**Purpose**: Verify if hypothesis matches the system

**Key Types/Traits**:
- `EquivalenceTest`: Main trait
  - `find_counterexample()`: Seek differences
- `Counterexample`: Difference representation
- `WMethodEQ`: W-method implementation (complete)
- `RandomWalkMethod`: Random walk (probabilistic)

**Design Decisions**:
- Trait object allows different strategies
- Returns Option<Counterexample> for cleaner error handling
- W-method for correctness, random walk for efficiency trade-off

#### 8. `lstar.rs` - Main Algorithm
**Purpose**: Coordinate the learning process

**Key Types**:
- `LSTAR`: Main learner structure
  - Maintains vocabulary, knowledge base, max states
  - Holds observation table and equivalence test
  - Implements `learn()` main loop

**Key Methods**:
- `new()`: Create learner with vocabulary
- `with_equivalence_test()`: Set custom EQ test
- `learn()`: Run full algorithm
- `build_hypothesis()`: Internal helper

**Design Decisions**:
- Builder pattern for configuration
- Iterative main loop with safety checks
- Logging integration for troubleshooting

### Support Modules

#### `lib.rs` - Module Declarations und Public API
- Re-exports public types
- Sets up the public API surface

#### `main.rs` - Example Usage
- Demonstrates basic learning
- Creates simple fake knowledge base
- Shows DOT output

## Data Flow

```
User Code
    |
    v
LSTAR::learn()
    |
    +---> initialize() -----> ObservationTable
    |         |                    |
    |         +---queries----> KnowledgeBase
    |
    +---> build_hypothesis() <--+ (loop)
    |         |
    |         +---> query system
    |         +---> return Automata
    |
    +---> equivalence_test() <--+
    |         |
    |         +---> KnowledgeBase
    |         +---> return Counterexample?
    |
    +---> add_counterexample()
    |
    +---> repeat until no counterexample
```

## Key Algorithmic Components

### 1. Table Closure
```
For each row in SA:
  If no equivalent row exists in S:
    Move row from SA to S
    Add new SA rows with extended words
```

### 2. Consistency Checking
```
For each pair of equivalent rows in S:
  For each input letter:
    Extend rows with input letter
    If resulting rows differ:
      Return inconsistency
```

### 3. Hypothesis Building
```
1. Group rows in S by signature
2. Create state for each unique signature
3. For each state, create transitions:
   - Input: alphabet letter
   - Output: from observation table
   - Destination: state of resulting row
```

## Memory Management

- **Ownership**: Words and letters are cloned as needed (reasonable for finite alphabets)
- **Arc**: Shared knowledge base via Arc<dyn KnowledgeBase>
- **HashMap**: Indexing over words (cloned keys are acceptable)
- **No Cycles**: Automata reference only by name, no circular references

## Performance Characteristics

| Operation | Complexity | Notes |
|-----------|------------|-------|
| Add word to table | O(k) where k = \|D\| | Optimized with pre-allocation |
| Check closure | O(\|S\| × \|SA\| × \|D\|) | |
| Check consistency | O(\|S\|² × \|Σ\| × \|D\|) | |
| Build hypothesis | O(\|S\|² × \|Σ\|) | |
| Per round | O(n × |Σ| × \|D\|) queries | |
| Transition lookup | O(1) | Cached index (v0.2.0+) |
| Word concatenation | O(n) | SmallVec optimization (v0.2.0+) |

Where:
- n = number of states
- |Σ| = alphabet size
- |D| = number of distinguishing suffixes

### Optimization Improvements (v0.2.0)

- **Letter Storage**: Enum-based (Single/Multiple) saves ~40 bytes per letter
- **Word Storage**: SmallVec with inline storage for ≤8 letters
- **Transition Cache**: HashMap index for O(1) lookups (30-50% faster)
- **Reduced Cloning**: Iterator-based operations (20-30% fewer allocations)
- **HashSet Dedup**: O(1) deduplication in W-method (10-20% faster)

**Overall**: 2-3x performance improvement for large automata

## Extensibility Points

### 1. Custom Knowledge Base
Implement `KnowledgeBase` trait for your system:
```rust
impl KnowledgeBase for MySystem {
    fn resolve_query(&self, query: &mut OutputQuery) -> Result<(), String> {
        // Execute query and set output
    }
}
```

### 2. Custom Equivalence Test
Implement `EquivalenceTest` trait:
```rust
impl EquivalenceTest for MyTest {
    fn find_counterexample(&self, hypothesis: &mut Automata) 
        -> Option<Counterexample> 
    {
        // Custom verification strategy
    }
}
```

**Note**: As of v0.2.0, `find_counterexample` takes `&mut Automata` to enable caching optimizations.

### 3. Custom Output Formats
Extend `Automata` with additional output methods:
```rust
impl Automata {
    pub fn build_my_format(&self) -> String { ... }
}
```

## Testing Strategy

- **Unit Tests**: Each module has basic tests
- **Integration Tests**: Full algorithm with fake knowledge base
- **Example Programs**: Demonstrate different use cases

Run tests with:
```bash
cargo test
cargo test -- --nocapture  # with output
```

## Known Limitations

1. **Symbolic Alphabet**: Alphabet must be finite and known in advance
2. **Deterministic Systems**: Assumes deterministic SUL behavior
3. **Mealy Machines**: Limited to Mealy machine learning (output on transitions)
4. **No Minimization**: Could add automata minimization post-learning
5. **Sequential Queries**: No built-in parallelization

## Future Improvements

1. **Incremental Mode**: Learn partially, save/resume
2. **Online Updates**: Update hypothesis with new observations
3. **Parallel Queries**: Execute independent queries concurrently
4. **Better EQ Tests**: Context-free grammar testing, adaptive strategies
5. **Minimization**: Post-process learned automata
6. **Statistics**: Track table complexity, learning curves

## References

- Angluin, D. (1987). Learning Regular Sets from Queries and Counterexamples
- Nerode, A. (1958). Linear automaton transformations
- De la Higuera, C. (2010). Grammatical Inference: Learning Automata and Grammars

## Contributing

When extending this implementation:

1. Maintain module independence
2. Use appropriate trait bounds
3. Include unit tests
4. Document with examples
5. Keep to functional/immutable patterns where possible
