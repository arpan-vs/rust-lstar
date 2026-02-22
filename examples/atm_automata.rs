/// Example: Build and display an automaton (ATM protocol)
use rust_lstar::{
    automata::{automata, transition},
    *,
};

fn add_transition(
    transitions: &mut Vec<Transition>,
    counter: &mut usize,
    src: &State,
    dst: &State,
    i: &Letter,
    o: &Letter,
) {
    transitions.push(transition::Transition::new_with_source(
        counter.to_string(),
        src.name.clone(),
        dst.clone(),
        i.clone(),
        o.clone(),
    ));
    *counter += 1;
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create states
    let mut states = vec![
        State::new("0".to_string()),
        State::new("1".to_string()),
        State::new("2".to_string()),
        State::new("3".to_string()),
        State::new("4".to_string()),
    ];

    // create vector of input letters
    let input_letters = vec![
        Letter::new("INSERT_CARD".to_string()),
        Letter::new("ENTER_PIN".to_string()),
        Letter::new("REQUEST_WITHDRAW".to_string()),
        Letter::new("EJECT_CARD".to_string()),
        Letter::new("TIMEOUT".to_string()),
    ];

    // create vector of output letter
    let output_letters = vec![
        Letter::new("CARD_ACCEPTED".to_string()),
        Letter::new("INVALID_OP".to_string()),
        Letter::new("RETRY".to_string()),
        Letter::new("PIN_VERIFIED".to_string()),
        Letter::new("CARD_EJECTED".to_string()),
        Letter::new("INVALID_COMMAND".to_string()),
        Letter::new("ENTER_AMOUNT".to_string()),
        Letter::new("DISPENSING".to_string()),
        Letter::new("WAIT".to_string()),
        Letter::new("SESSION_TIMEOUT".to_string()),
    ];

    // create vector of transitions
    let mut transitions: Vec<Transition> = Vec::new();
    let mut counter: usize = 0;
    // Transitions (from the provided DOT)
    add_transition(
        &mut transitions,
        &mut counter,
        &states[0],
        &states[1],
        &input_letters[0],
        &output_letters[0],
    );
    add_transition(
        &mut transitions,
        &mut counter,
        &states[0],
        &states[0],
        &input_letters[1],
        &output_letters[3],
    );
    add_transition(
        &mut transitions,
        &mut counter,
        &states[0],
        &states[0],
        &input_letters[2],
        &output_letters[1],
    );
    add_transition(
        &mut transitions,
        &mut counter,
        &states[0],
        &states[0],
        &input_letters[3],
        &output_letters[2],
    );
    add_transition(
        &mut transitions,
        &mut counter,
        &states[0],
        &states[0],
        &input_letters[4],
        &output_letters[1],
    );
    states[0].transitions = transitions[0..5].to_vec();

    add_transition(
        &mut transitions,
        &mut counter,
        &states[1],
        &states[1],
        &input_letters[0],
        &output_letters[2],
    );
    add_transition(
        &mut transitions,
        &mut counter,
        &states[1],
        &states[2],
        &input_letters[1],
        &output_letters[3],
    );
    add_transition(
        &mut transitions,
        &mut counter,
        &states[1],
        &states[1],
        &input_letters[2],
        &output_letters[2],
    );
    add_transition(
        &mut transitions,
        &mut counter,
        &states[1],
        &states[0],
        &input_letters[3],
        &output_letters[4],
    );
    add_transition(
        &mut transitions,
        &mut counter,
        &states[1],
        &states[1],
        &input_letters[4],
        &output_letters[2],
    );
    states[1].transitions = transitions[5..10].to_vec();

    add_transition(
        &mut transitions,
        &mut counter,
        &states[2],
        &states[2],
        &input_letters[0],
        &output_letters[5],
    );
    add_transition(
        &mut transitions,
        &mut counter,
        &states[2],
        &states[2],
        &input_letters[1],
        &output_letters[5],
    );
    add_transition(
        &mut transitions,
        &mut counter,
        &states[2],
        &states[3],
        &input_letters[2],
        &output_letters[6],
    );
    add_transition(
        &mut transitions,
        &mut counter,
        &states[2],
        &states[0],
        &input_letters[3],
        &output_letters[4],
    );
    add_transition(
        &mut transitions,
        &mut counter,
        &states[2],
        &states[0],
        &input_letters[4],
        &output_letters[9],
    );
    states[2].transitions = transitions[10..15].to_vec();

    add_transition(
        &mut transitions,
        &mut counter,
        &states[3],
        &states[3],
        &input_letters[0],
        &output_letters[8],
    );
    add_transition(
        &mut transitions,
        &mut counter,
        &states[3],
        &states[3],
        &input_letters[1],
        &output_letters[8],
    );
    add_transition(
        &mut transitions,
        &mut counter,
        &states[3],
        &states[4],
        &input_letters[2],
        &output_letters[7],
    );
    add_transition(
        &mut transitions,
        &mut counter,
        &states[3],
        &states[0],
        &input_letters[3],
        &output_letters[4],
    );
    add_transition(
        &mut transitions,
        &mut counter,
        &states[3],
        &states[3],
        &input_letters[4],
        &output_letters[7],
    );
    states[3].transitions = transitions[15..20].to_vec();

    add_transition(
        &mut transitions,
        &mut counter,
        &states[4],
        &states[4],
        &input_letters[0],
        &output_letters[7],
    );
    add_transition(
        &mut transitions,
        &mut counter,
        &states[4],
        &states[4],
        &input_letters[1],
        &output_letters[7],
    );
    add_transition(
        &mut transitions,
        &mut counter,
        &states[4],
        &states[4],
        &input_letters[2],
        &output_letters[7],
    );
    add_transition(
        &mut transitions,
        &mut counter,
        &states[4],
        &states[0],
        &input_letters[3],
        &output_letters[4],
    );
    add_transition(
        &mut transitions,
        &mut counter,
        &states[4],
        &states[4],
        &input_letters[4],
        &output_letters[7],
    );
    states[4].transitions = transitions[20..25].to_vec();

    // add("0", "1", "INSERT_CARD", "CARD_ACCEPTED");
    // add("0", "0", "ENTER_PIN", "INVALID_OP");
    // add("0", "0", "REQUEST_WITHDRAW", "INVALID_OP");
    // add("0", "0", "EJECT_CARD", "INVALID_OP");
    // add("0", "0", "TIMEOUT", "INVALID_OP");

    // add("1", "1", "INSERT_CARD", "RETRY");
    // add("1", "2", "ENTER_PIN", "PIN_VERIFIED");
    // add("1", "1", "REQUEST_WITHDRAW", "RETRY");
    // add("1", "0", "EJECT_CARD", "CARD_EJECTED");
    // add("1", "1", "TIMEOUT", "RETRY");

    // add("4", "4", "INSERT_CARD", "DISPENSING");
    // add("4", "4", "ENTER_PIN", "DISPENSING");
    // add("4", "4", "REQUEST_WITHDRAW", "DISPENSING");
    // add("4", "0", "EJECT_CARD", "CARD_EJECTED");
    // add("4", "4", "TIMEOUT", "DISPENSING");

    // add("2", "2", "INSERT_CARD", "INVALID_COMMAND");
    // add("2", "2", "ENTER_PIN", "INVALID_COMMAND");
    // add("2", "3", "REQUEST_WITHDRAW", "ENTER_AMOUNT");
    // add("2", "0", "EJECT_CARD", "CARD_EJECTED");
    // add("2", "0", "TIMEOUT", "SESSION_TIMEOUT");

    // add("3", "3", "INSERT_CARD", "WAIT");
    // add("3", "3", "ENTER_PIN", "WAIT");
    // add("3", "4", "REQUEST_WITHDRAW", "DISPENSING");
    // add("3", "0", "EJECT_CARD", "CARD_EJECTED");
    // add("3", "3", "TIMEOUT", "WAIT");

    let mut automaton = automata::Automata::new(states[0].clone(), "ATM".to_string());
    automaton.transitions = transitions.clone();
    // Print DOT
    println!("{}", automaton.build_dot_code());

    Ok(())
}
