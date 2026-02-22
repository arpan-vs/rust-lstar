//! Compare multiple equivalence-test strategies on the same custom ATM-like
//! system under learning and print runtime/statistics summaries.

use rust_lstar::eqtest::{BDistMethod, MultipleEqtests, RandomWalkMethod, WMethodEQ};
use rust_lstar::knowledge_base::{KnowledgeBaseStats, KnowledgeBaseTrait};
use rust_lstar::query::OutputQuery;
use rust_lstar::*;
use std::sync::{Arc, Mutex};
use std::time::Instant;

type SharedKb = Arc<Mutex<dyn KnowledgeBaseTrait>>;
type EqBuilder = dyn Fn(SharedKb, Vec<Letter>, usize) -> Arc<dyn EquivalenceTest>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Custom KB: All Equivalence Tests ===");
    println!("System under learning: ATM protocol");

    let mut reports = Vec::new();

    let strategies: Vec<(&str, Box<EqBuilder>)> = vec![
        (
            "WMethodEQ",
            Box::new(|kb, input_letters, max_states| {
                Arc::new(WMethodEQ::new(kb, input_letters, max_states))
            }),
        ),
        (
            "RandomWalkMethod",
            Box::new(|kb, input_letters, _| {
                Arc::new(RandomWalkMethod::new(kb, input_letters, 10_000, 0.75))
            }),
        ),
        (
            "BDistMethod",
            Box::new(|kb, input_letters, _| Arc::new(BDistMethod::new(kb, input_letters, 2))),
        ),
        (
            "MultipleEqtests",
            Box::new(|kb, input_letters, max_states| {
                let eqtests: Vec<Arc<dyn EquivalenceTest>> = vec![
                    Arc::new(WMethodEQ::new(
                        kb.clone(),
                        input_letters.clone(),
                        max_states,
                    )),
                    Arc::new(RandomWalkMethod::new(
                        kb.clone(),
                        input_letters.clone(),
                        5_000,
                        0.65,
                    )),
                    Arc::new(BDistMethod::new(kb, input_letters, 2)),
                ];
                Arc::new(MultipleEqtests::new(eqtests))
            }),
        ),
    ];

    for (name, builder) in strategies {
        reports.push(run_strategy(name, builder.as_ref()));
    }

    print_summary(&reports);

    Ok(())
}

fn run_strategy(name: &str, builder: &EqBuilder) -> StrategyReport {
    let vocabulary = vec![
        "INSERT_CARD".to_string(),
        "ENTER_PIN".to_string(),
        "REQUEST_WITHDRAW".to_string(),
        "EJECT_CARD".to_string(),
        "TIMEOUT".to_string(),
    ];
    let input_letters = vocabulary
        .iter()
        .map(|s| Letter::new(s))
        .collect::<Vec<_>>();
    let max_states = 8;

    let kb = Arc::new(Mutex::new(ATMKnowledgeBase::new()));
    let knowledge_base: SharedKb = kb.clone();
    let eqtest = builder(knowledge_base.clone(), input_letters, max_states);
    let mut learner = LSTAR::new(vocabulary, knowledge_base, max_states, None, Some(eqtest));

    println!("\n--- Running {name} ---");
    let started = Instant::now();
    let learn_result = learner.learn();
    let elapsed_ms = started.elapsed().as_millis();

    let stats = {
        let kb_guard = kb.lock().unwrap();
        StatsSnapshot::from_stats(&kb_guard.stats)
    };

    match learn_result {
        Ok(automata) => {
            let state_count = automata.get_states().len();
            let transition_count = automata.transitions.len();
            println!("{name}: success (states={state_count}, transitions={transition_count})");
            StrategyReport {
                name: name.to_string(),
                elapsed_ms,
                state_count: Some(state_count),
                transition_count: Some(transition_count),
                error: None,
                stats,
            }
        }
        Err(err) => {
            println!("{name}: failed ({err})");
            StrategyReport {
                name: name.to_string(),
                elapsed_ms,
                state_count: None,
                transition_count: None,
                error: Some(err),
                stats,
            }
        }
    }
}

fn print_summary(reports: &[StrategyReport]) {
    println!("\n=== Final Statistics Summary ===");
    for report in reports {
        println!("\n[{}]", report.name);
        println!("  runtime_ms: {}", report.elapsed_ms);
        match (&report.state_count, &report.transition_count) {
            (Some(states), Some(transitions)) => {
                println!("  states: {}", states);
                println!("  transitions: {}", transitions);
            }
            _ => println!("  model: n/a"),
        }
        match &report.error {
            Some(err) => println!("  status: failed ({err})"),
            None => println!("  status: success"),
        }
        println!("  kb_nb_query: {}", report.stats.nb_query);
        println!(
            "  kb_nb_submitted_query: {}",
            report.stats.nb_submitted_query
        );
        println!("  kb_nb_letter: {}", report.stats.nb_letter);
        println!(
            "  kb_nb_submitted_letter: {}",
            report.stats.nb_submitted_letter
        );
    }
}

/// Minimal ATM simulator used as a custom knowledge base.
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
        Self {
            state: ATMState::Idle,
            stats: KnowledgeBaseStats::new(),
        }
    }

    fn process_input(&self, command: &str, current_state: ATMState) -> (ATMState, &'static str) {
        match current_state {
            ATMState::Idle => match command {
                "INSERT_CARD" => (ATMState::CardInserted, "CARD_ACCEPTED"),
                _ => (ATMState::Idle, "INVALID_OP"),
            },
            ATMState::CardInserted => match command {
                "ENTER_PIN" => (ATMState::Authenticated, "PIN_VERIFIED"),
                "EJECT_CARD" => (ATMState::Idle, "CARD_EJECTED"),
                _ => (ATMState::CardInserted, "RETRY"),
            },
            ATMState::Authenticated => match command {
                "REQUEST_WITHDRAW" => (ATMState::Ready, "ENTER_AMOUNT"),
                "EJECT_CARD" => (ATMState::Idle, "CARD_EJECTED"),
                "TIMEOUT" => (ATMState::Idle, "SESSION_TIMEOUT"),
                _ => (ATMState::Authenticated, "INVALID_COMMAND"),
            },
            ATMState::Ready => match command {
                "REQUEST_WITHDRAW" => (ATMState::Dispensing, "DISPENSING"),
                "EJECT_CARD" => (ATMState::Idle, "CARD_EJECTED"),
                _ => (ATMState::Ready, "WAIT"),
            },
            ATMState::Dispensing => match command {
                "EJECT_CARD" => (ATMState::Idle, "CARD_EJECTED"),
                _ => (ATMState::Dispensing, "DISPENSING"),
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

        self.state = ATMState::Idle;
        let mut outputs = Vec::new();

        for input_letter in query.input_word.letters() {
            let command = input_letter.symbols();
            let (next_state, response) = self.process_input(command.as_str(), self.state);
            self.state = next_state;
            outputs.push(Letter::new(response));
        }

        query.set_result(Word::from_letters(outputs));
        Ok(())
    }

    fn add_word(&mut self, _input: &Word, _output: &Word) -> Result<(), String> {
        Ok(())
    }
}

/// Snapshot of key knowledge-base counters for final reporting.
struct StatsSnapshot {
    nb_query: usize,
    nb_submitted_query: usize,
    nb_letter: usize,
    nb_submitted_letter: usize,
}

impl StatsSnapshot {
    fn from_stats(stats: &KnowledgeBaseStats) -> Self {
        Self {
            nb_query: stats.nb_query(),
            nb_submitted_query: stats.nb_submitted_query(),
            nb_letter: stats.nb_letter(),
            nb_submitted_letter: stats.nb_submitted_letter(),
        }
    }
}

/// Summary row for one equivalence-test strategy execution.
struct StrategyReport {
    name: String,
    elapsed_ms: u128,
    state_count: Option<usize>,
    transition_count: Option<usize>,
    error: Option<String>,
    stats: StatsSnapshot,
}
