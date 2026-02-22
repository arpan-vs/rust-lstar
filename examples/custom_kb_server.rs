//! TCP server exposing the ATM protocol used by `custom_kb` examples.
//!
//! It mirrors `examples/coffee_server.rs` but serves the ATM-like state
//! machine so it can be learned via `NetworkActiveKnowledgeBase`.

use std::env;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

#[derive(Clone, Copy)]
enum ATMState {
    Idle,
    CardInserted,
    Authenticated,
    Ready,
    Dispensing,
}

/// Stateful ATM simulator used as server-side SUL.
struct ATMServerMachine {
    state: ATMState,
}

impl ATMServerMachine {
    fn new() -> Self {
        Self {
            state: ATMState::Idle,
        }
    }

    fn process_input(command: &str, current_state: ATMState) -> (ATMState, &'static str) {
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

    fn execute_command(&mut self, command: &str) -> &'static str {
        let (next_state, response) = Self::process_input(command, self.state);
        self.state = next_state;
        response
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Optional CLI arg: address (default 127.0.0.1:3001)
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:3001".to_string());
    println!("Custom KB ATM server listening on {}", addr);

    let listener = TcpListener::bind(&addr)?;

    // Serve clients concurrently; keep the server running until killed.
    for stream in listener.incoming() {
        match stream {
            Ok(mut s) => {
                let peer = s.peer_addr()?;
                println!("Client connected: {}", peer);
                thread::spawn(move || {
                    if let Err(e) = handle_client(&mut s) {
                        eprintln!("Client {} error: {}", peer, e);
                    }
                    println!("Client {} disconnected", peer);
                });
            }
            Err(e) => eprintln!("Accept error: {}", e),
        }
    }

    Ok(())
}

fn handle_client(stream: &mut TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let mut machine = ATMServerMachine::new();
    let mut buf = [0u8; 1024];

    loop {
        let n = stream.read(&mut buf)?;
        if n == 0 {
            break; // client closed
        }

        let req = String::from_utf8_lossy(&buf[..n]);
        let trimmed = req.trim();
        println!("Received request: '{}'", trimmed);
        let response = machine.execute_command(normalize_request(trimmed));
        stream.write_all(response.as_bytes())?;
    }

    Ok(())
}

fn normalize_request(req: &str) -> &str {
    req.strip_prefix("Letter('")
        .and_then(|s| s.strip_suffix("')"))
        .unwrap_or(req)
}
