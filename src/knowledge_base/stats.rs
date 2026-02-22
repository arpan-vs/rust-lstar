/// Knowledge base statistics module
/// Tracks queries and letters submitted during learning

pub struct KnowledgeBaseStats {
    nb_query: usize,
    nb_submitted_query: usize,
    nb_letter: usize,
    nb_submitted_letter: usize,
}

impl KnowledgeBaseStats {
    /// Creates a new KnowledgeBaseStats instance with all counters at 0
    pub fn new() -> Self {
        KnowledgeBaseStats {
            nb_query: 0,
            nb_submitted_query: 0,
            nb_letter: 0,
            nb_submitted_letter: 0,
        }
    }

    /// Gets the number of queries submitted to the target
    pub fn nb_submitted_query(&self) -> usize {
        self.nb_submitted_query
    }

    /// Sets the number of queries submitted to the target
    pub fn set_nb_submitted_query(&mut self, value: usize) {
        self.nb_submitted_query = value;
    }

    /// Increments the number of queries submitted
    pub fn increment_nb_submitted_query(&mut self) {
        self.nb_submitted_query += 1;
    }

    /// Gets the number of letters submitted to the target
    pub fn nb_submitted_letter(&self) -> usize {
        self.nb_submitted_letter
    }

    /// Sets the number of letters submitted to the target
    pub fn set_nb_submitted_letter(&mut self, value: usize) {
        self.nb_submitted_letter = value;
    }

    /// Adds to the number of letters submitted
    pub fn add_nb_submitted_letter(&mut self, count: usize) {
        self.nb_submitted_letter += count;
    }

    /// Gets the total number of letters triggered while inferring
    pub fn nb_letter(&self) -> usize {
        self.nb_letter
    }

    /// Sets the total number of letters triggered
    pub fn set_nb_letter(&mut self, value: usize) {
        self.nb_letter = value;
    }

    /// Adds to the total number of letters triggered
    pub fn add_nb_letter(&mut self, count: usize) {
        self.nb_letter += count;
    }

    /// Gets the number of queries triggered while inferring
    pub fn nb_query(&self) -> usize {
        self.nb_query
    }

    /// Sets the number of queries triggered
    pub fn set_nb_query(&mut self, value: usize) {
        self.nb_query = value;
    }

    /// Increments the number of queries triggered
    pub fn increment_nb_query(&mut self) {
        self.nb_query += 1;
    }
}

impl Default for KnowledgeBaseStats {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for KnowledgeBaseStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "\t- nb query = {}\n\t- nb submitted query = {}\n\t- nb letter = {}\n\t- nb submitted letter = {}\n",
            self.nb_query, self.nb_submitted_query, self.nb_letter, self.nb_submitted_letter
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creation() {
        let stats = KnowledgeBaseStats::new();
        assert_eq!(stats.nb_query(), 0);
        assert_eq!(stats.nb_submitted_query(), 0);
        assert_eq!(stats.nb_letter(), 0);
        assert_eq!(stats.nb_submitted_letter(), 0);
    }

    #[test]
    fn test_increment_query() {
        let mut stats = KnowledgeBaseStats::new();
        stats.increment_nb_query();
        stats.increment_nb_query();
        assert_eq!(stats.nb_query(), 2);
    }

    #[test]
    fn test_add_letter() {
        let mut stats = KnowledgeBaseStats::new();
        stats.add_nb_letter(5);
        stats.add_nb_letter(3);
        assert_eq!(stats.nb_letter(), 8);
    }
}
