use std::vec;

use rust_lstar::automata::{State, Transition};
use rust_lstar::letter::Letter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut s1 = State::new("s1".to_string());
    let mut s2 = State::new("s2".to_string());

    let a = Letter::new('a');
    let b = Letter::new('b');
    let o1 = Letter::new('1');
    let o2 = Letter::new('2');
    
    let t1 = Transition::new(
        "t1".to_string(),
        s1.clone(),
        s2.clone(),
        a.clone(),
        o1.clone()
    );

    let t2 = Transition::new(
        "t2".to_string(),
        s1.clone(),
        s1.clone(),
        b.clone(),
        o2.clone()
    );
    s1.transitions = vec![t1.clone(), t2.clone()];

    let t3 = Transition::new(
        "t3".to_string(),
        s2.clone(),
        s2.clone(),
        b.clone(),
        o2.clone()
    );

    let t4 = Transition::new(
        "t4".to_string(),
        s2.clone(),
        s1.clone(),
        a.clone(),
        o1.clone()
    );
    s2.transitions = vec![t3.clone(), t4.clone()];

    println!("Transition 1: {}", t1);
    println!("Transition 2: {}", t2);
    println!("Transition 3: {}", t3);
    println!("Transition 4: {}", t4);

    let ans = s1.visit(&a);

    if let Some((state, letter)) = ans {
        println!("State {} visited by {:?} goes to state {}, output symbol {:?}", s1, a, state, letter);
    } else {
        panic!();
    }

    Ok(())
}
