use bio::alignment::Alignment;
use bio::pattern_matching::myers::MyersBuilder;

/// Search pattern structure
#[derive(Debug, Clone)]
pub struct SearchPattern {
    /// Raw text
    pub raw_text: Vec<u8>,
    /// Search text
    pub text: Vec<u8>,
    /// Raw text length
    pub raw_text_len: usize,
    /// Search pattern
    pub pattern: Vec<u8>,
    /// Distance ratio
    pub dist_ratio: f32,
    /// Maximum distance
    pub max_dist: u8,
    /// Start position
    pub start: usize,
    /// End position
    pub end: usize,
}

impl SearchPattern {
    /// Create a new search pattern
    pub fn new(raw_text: Vec<u8>, distance_ratio: f32) -> Self {
        Self {
            raw_text: raw_text.clone(),
            text: Vec::new(),
            raw_text_len: raw_text.len(),
            pattern: Vec::new(),
            dist_ratio: distance_ratio,
            max_dist: 0,
            start: 0,
            end: 0,
        }
    }
    
    /// Update search parameters
    pub fn update(&mut self, start_position: usize, end_position: usize, pattern: Vec<u8>) {
        // Calculate pattern length after trimming N
        let trimmed_pattern_length = String::from_utf8(pattern.clone())
            .unwrap()
            .trim_matches('N')
            .len() as f32;
        
        // Calculate maximum distance
        self.max_dist = (trimmed_pattern_length * self.dist_ratio).floor() as u8;
        self.start = start_position;
        self.end = end_position;
        self.text = self.raw_text[self.start..self.end].to_vec();
        self.pattern = pattern;
    }
    
    /// Get search text
    pub fn get_search_text(&self) -> &[u8] {
        &self.text
    }
    
    
    /// Get maximum distance
    pub fn get_max_distance(&self) -> u8 {
        self.max_dist
    }
    
    /// Get start position
    pub fn get_start_position(&self) -> usize {
        self.start
    }
    
}

/// Perform best match search using Myers algorithm
pub fn myers_best(search_pattern: &SearchPattern) -> Option<(i32, usize, usize)> {
    // Create Myers builder for fuzzy matching
    let mut myers = MyersBuilder::new()
        .ambig(b'N', b"ACGT")
        .build_64(search_pattern.pattern.clone());
    
    let mut alignment = Alignment::default();
    let mut matches = myers.find_all_lazy(search_pattern.get_search_text(), search_pattern.get_max_distance());
    
    // Find the best match
    match matches.by_ref().min_by_key(|&(_, distance)| distance) {
        Some((best_end, _)) => {
            matches.alignment_at(best_end, &mut alignment);
            Some((
                alignment.score,
                alignment.ystart + search_pattern.get_start_position(),
                alignment.yend + search_pattern.get_start_position(),
            ))
        }
        None => None,
    }
}




#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_search_pattern_creation() {
        let raw_text = b"ATCGATCG".to_vec();
        let search_pattern = SearchPattern::new(raw_text, 0.1);
        
        assert_eq!(search_pattern.raw_text_len, 8);
        assert_eq!(search_pattern.dist_ratio, 0.1);
    }
    
}