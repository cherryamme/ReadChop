use log::info;

/// Calculate the reverse complement of a DNA sequence
pub fn reverse_complement(sequence: &str) -> String {
    let mut complement = vec![' '; sequence.len()];
    
    for (i, nucleotide) in sequence.chars().enumerate() {
        complement[sequence.len() - 1 - i] = match nucleotide {
            'A' => 'T',
            'T' => 'A',
            'C' => 'G',
            'G' => 'C',
            'a' => 't',
            't' => 'a',
            'c' => 'g',
            'g' => 'c',
            _ => panic!("Invalid nucleotide character: {}", nucleotide),
        };
    }
    
    complement.into_iter().collect::<String>()
}

/// Process information tracker
pub struct ProcessInfo {
    start_time: std::time::Instant,
    end_time: std::time::Instant,
    processed_count: u32,
    log_interval: u32,
}

impl ProcessInfo {
    /// Create a new process information tracker
    pub fn new(log_interval: u32) -> Self {
        Self {
            start_time: std::time::Instant::now(),
            end_time: std::time::Instant::now(),
            processed_count: 0,
            log_interval,
        }
    }
    
    /// Update process information
    pub fn info(&mut self) {
        self.processed_count += 1;
        
        if self.processed_count % self.log_interval == 0 {
            self.end_time = std::time::Instant::now();
            let elapsed = self.end_time.duration_since(self.start_time);
            let processing_rate = self.processed_count as f64 / elapsed.as_secs_f64();
            
            info!(
                "Processed {} sequences, processing speed: {:.2} sequences/second", 
                self.processed_count, 
                processing_rate
            );
            
            self.start_time = std::time::Instant::now();
            self.processed_count = 0;
        }
    }
    
}
