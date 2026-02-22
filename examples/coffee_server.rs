//! TCP server used by network knowledge-base examples.
//!
//! It exposes a simple line-oriented command interface for a coffee machine
//! state machine.

use std::env;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

/// Stateful coffee-machine simulator used as server-side SUL.
struct CoffeeMachine {
    qte_water: usize,
    qte_coffee: usize,
    switch_on: bool,
}

impl CoffeeMachine {
    const MAX_WATER: usize = 3;
    const MAX_COFFEE: usize = 3;

    fn new() -> Self {
        CoffeeMachine {
            qte_water: 0,
            qte_coffee: 0,
            switch_on: false,
        }
    }

    fn refill_water(&mut self) -> Option<&'static str> {
        if self.qte_water == Self::MAX_WATER {
            return None;
        }
        self.qte_water = Self::MAX_WATER;
        Some("DONE")
    }

    fn refill_coffee(&mut self) -> Option<&'static str> {
        if self.qte_coffee == Self::MAX_COFFEE {
            return None;
        }
        self.qte_coffee = Self::MAX_COFFEE;
        Some("DONE")
    }

    fn press_button_a(&mut self) -> Option<&'static str> {
        if !self.switch_on {
            return None;
        }
        if self.qte_water == 0 {
            return None;
        }
        if self.qte_coffee == 0 {
            return None;
        }
        self.qte_water -= 1;
        self.qte_coffee -= 1;
        Some("NORMAL COFFEE IS SERVED")
    }

    fn press_button_b(&mut self) -> Option<&'static str> {
        if !self.switch_on {
            return None;
        }
        if self.qte_water == 0 {
            return None;
        }
        if self.qte_coffee <= 2 {
            return None;
        }
        self.qte_water -= 1;
        self.qte_coffee -= 2;
        Some("EXPRESSO IS SERVED")
    }

    fn press_button_c(&mut self) -> &'static str {
        self.switch_on = !self.switch_on;
        if !self.switch_on {
            "COFFEE MACHINE IS OFF"
        } else {
            "COFFEE MACHINE IS ON"
        }
    }

    fn execute_command(&mut self, cmd: &str) -> &'static str {
        let c = cmd.trim();
        if c.is_empty() {
            return "ERROR";
        }

        match c {
            "REFILL_WATER" => self.refill_water().unwrap_or("ERROR"),
            "REFILL_COFFEE" => self.refill_coffee().unwrap_or("ERROR"),
            "PRESS_BUTTON_A" => self.press_button_a().unwrap_or("ERROR"),
            "PRESS_BUTTON_B" => self.press_button_b().unwrap_or("ERROR"),
            "PRESS_BUTTON_C" => self.press_button_c(),
            _ => "ERROR",
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Optional CLI arg: address (default 127.0.0.1:3000)
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:3000".to_string());
    println!("Coffee server listening on {}", addr);

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
    let mut machine = CoffeeMachine::new();
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
