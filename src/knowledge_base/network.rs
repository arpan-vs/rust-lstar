use super::active::ActiveKnowledgeBase;
use super::base::{KnowledgeBase, KnowledgeBaseTrait};
use super::stats::KnowledgeBaseStats;
use crate::letter::Letter;
use crate::query::OutputQuery;
/// Network active knowledge base implementation
/// Communicates with a remote target via network sockets
use crate::word::Word;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

/// An active knowledge base that communicates with a remote target via network
pub struct NetworkActiveKnowledgeBase {
    base: KnowledgeBase,
    target_host: String,
    target_port: u16,
    timeout: Duration,
    target_running: bool,
}

impl NetworkActiveKnowledgeBase {
    /// Creates a new network knowledge base
    ///
    /// # Arguments
    /// * `target_host` - Hostname or IP address of the target
    /// * `target_port` - Port number of the target service
    /// * `timeout` - Socket timeout duration
    pub fn new(target_host: String, target_port: u16, timeout: Duration) -> Self {
        NetworkActiveKnowledgeBase {
            base: KnowledgeBase::new(),
            target_host,
            target_port,
            timeout,
            target_running: false,
        }
    }

    /// Gets the target hostname
    pub fn target_host(&self) -> &str {
        &self.target_host
    }

    /// Gets the target port
    pub fn target_port(&self) -> u16 {
        self.target_port
    }

    /// Gets the socket timeout
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Sets the socket timeout
    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }

    /// Gets the connection address
    fn connection_addr(&self) -> String {
        format!("{}:{}", self.target_host, self.target_port)
    }

    /// Sends data to the target and receives response
    fn send_and_receive(&self, stream: &mut TcpStream, data: &[u8]) -> Result<Vec<u8>, String> {
        // Send data
        stream
            .write_all(data)
            .map_err(|e| format!("Failed to send data: {}", e))?;

        // Receive response
        let mut buffer = vec![0; 1024];
        let n = stream
            .read(&mut buffer)
            .map_err(|e| format!("Failed to receive data: {}", e))?;

        buffer.truncate(n);
        Ok(buffer)
    }

    /// Submits a single letter to the network target
    fn submit_letter(&self, stream: &mut TcpStream, letter: &Letter) -> Result<Letter, String> {
        let data = letter.symbols().into_bytes();
        let response = self.send_and_receive(stream, &data)?;

        match String::from_utf8(response) {
            Ok(response_str) => {
                let trimmed = response_str.trim();
                Ok(Letter::new(trimmed))
            }
            Err(_) => Ok(Letter::new("")),
        }
    }
}

impl ActiveKnowledgeBase for NetworkActiveKnowledgeBase {
    fn start_target(&mut self) -> Result<(), String> {
        // The underlying protocol is queried in submit_word() with a fresh
        // connection for each word (same behavior as pylstar).
        self.target_running = true;
        Ok(())
    }

    fn stop_target(&mut self) -> Result<(), String> {
        self.target_running = false;
        Ok(())
    }

    fn submit_word(&mut self, word: &Word) -> Result<Word, String> {
        let addr = self.connection_addr();
        let mut stream = TcpStream::connect(&addr)
            .map_err(|e| format!("Failed to connect to {}: {}", addr, e))?;

        stream
            .set_read_timeout(Some(self.timeout))
            .map_err(|e| format!("Failed to set read timeout: {}", e))?;
        stream
            .set_write_timeout(Some(self.timeout))
            .map_err(|e| format!("Failed to set write timeout: {}", e))?;

        let mut output_letters = Vec::new();

        for letter in word.letters() {
            output_letters.push(self.submit_letter(&mut stream, letter)?);
        }

        Ok(Word::from_letters(output_letters))
    }

    fn is_target_running(&self) -> bool {
        self.target_running
    }
}

impl KnowledgeBaseTrait for NetworkActiveKnowledgeBase {
    fn resolve_query(&mut self, query: &mut OutputQuery) -> Result<(), String> {
        match self.base.resolve_query(query) {
            Ok(_) => Ok(()),
            Err(_) => {
                self.start_target()?;
                let submit_result = self.submit_word(&query.input_word);
                let stop_result = self.stop_target();

                let output = submit_result?;
                stop_result?;

                self.base.add_word(&query.input_word, &output)?;
                query.set_result(output);
                Ok(())
            }
        }
    }

    fn add_word(&mut self, input_word: &Word, output_word: &Word) -> Result<(), String> {
        self.base.add_word(input_word, output_word)
    }
}

impl NetworkActiveKnowledgeBase {
    pub fn stats(&self) -> &KnowledgeBaseStats {
        self.base.stats()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creation() {
        let kb =
            NetworkActiveKnowledgeBase::new("localhost".to_string(), 3000, Duration::from_secs(5));

        assert_eq!(kb.target_host(), "localhost");
        assert_eq!(kb.target_port(), 3000);
        assert!(!kb.is_target_running());
    }

    #[test]
    fn test_connection_addr() {
        let kb = NetworkActiveKnowledgeBase::new(
            "example.com".to_string(),
            8080,
            Duration::from_secs(5),
        );

        assert_eq!(kb.connection_addr(), "example.com:8080");
    }

    #[test]
    fn test_set_timeout() {
        let mut kb =
            NetworkActiveKnowledgeBase::new("localhost".to_string(), 3000, Duration::from_secs(5));

        assert_eq!(kb.timeout(), Duration::from_secs(5));
        kb.set_timeout(Duration::from_secs(10));
        assert_eq!(kb.timeout(), Duration::from_secs(10));
    }
}
