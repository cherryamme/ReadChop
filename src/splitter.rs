use crate::fastq::ReadInfo;
use crate::myers::myers_best;
use crate::myers::SearchPattern;
use crate::pattern::{PatternArgument, PatternConfiguration};
use crate::thread_pool::ThreadPoolManager;
use bio::io::fastq::Record;
use flume::Receiver;
use std::cmp::min;
use std::collections::HashMap;
use std::thread;
use std::time::Instant;

/// Read block structure for defining search range
#[derive(Debug)]
struct ReadChunk {
    left_bound: usize,
    right_bound: usize,
    use_position_mutation: bool,
}

impl ReadChunk {
    /// Create new read block
    pub fn new(pattern_config: &PatternConfiguration, read_info: &ReadInfo) -> Self {
        let left_bound = if pattern_config.window_size[0] > read_info.sequence_length {
            read_info.sequence_length
        } else {
            pattern_config.window_size[0]
        };

        let right_bound = if pattern_config.window_size[1] > read_info.sequence_length {
            0
        } else {
            read_info.sequence_length - pattern_config.window_size[1]
        };

        Self {
            left_bound,
            right_bound,
            use_position_mutation: false,
        }
    }
}

/// Split type structure
#[derive(Debug, Clone)]
pub struct SplitType {
    pub pattern_match: &'static str, // single or dual
    pub pattern_name: String,         // pattern name ex:4.2-F_3.7-R
    pub pattern_type: String,        // pattern type ex:alpha
    pub pattern_strand: String,      // strand orientation
    pub left_matcher: Matcher,        // left matcher
    pub right_matcher: Matcher,      // right matcher
}

impl SplitType {
    /// Create new split type
    pub fn new(left_matcher: Matcher, right_matcher: Matcher) -> Self {
        Self {
            pattern_match: "unknown",
            pattern_name: String::from("unknown"),
            pattern_type: String::from("unknown"),
            pattern_strand: String::from("unknown"),
            left_matcher,
            right_matcher,
        }
    }
    
    /// Convert to information string
    pub fn to_info(&self) -> String {
        format!(
            "{}\t{}\t{}\t{}:({},{},{},{});({},{},{},{})",
            self.pattern_match,
            self.pattern_name,
            self.pattern_type,
            self.pattern_strand,
            self.left_matcher.pattern,
            self.left_matcher.score,
            self.left_matcher.ystart,
            self.left_matcher.yend,
            self.right_matcher.pattern,
            self.right_matcher.score,
            self.right_matcher.ystart,
            self.right_matcher.yend,
        )
    }
    
    /// Annotate pattern type
    pub fn annotate_pattern_type(
        &mut self,
        pattern_type_dict: &HashMap<String, (String, String, String)>,
        max_distance: i32,
    ) {
        let (pattern_match, key) = self.get_match_key(max_distance, pattern_type_dict);
        
        if key == "_" || key == "unknown" {
            return;
        }
        
        for (dict_key, value) in pattern_type_dict {
            if dict_key.contains(&key) {
                self.pattern_match = pattern_match;
                self.pattern_name = value.0.clone();
                self.pattern_type = value.1.clone();
                self.pattern_strand = value.2.clone();
                break;
            }
        }
    }
    
    /// Get match key
    pub fn get_match_key(
        &self,
        max_distance: i32,
        pattern_type_dict: &HashMap<String, (String, String, String)>,
    ) -> (&'static str, String) {
        if self.right_matcher.status && self.left_matcher.status {
            let combined_pattern = format!("{}_{}", self.left_matcher.pattern, self.right_matcher.pattern);
            if pattern_type_dict.contains_key(&combined_pattern) {
                return ("dual", combined_pattern);
            }
            let score_difference = self.right_matcher.score - self.left_matcher.score;
            if score_difference.abs() <= max_distance {
                return ("dual", combined_pattern);
            }
            if score_difference > 0 {
                ("left", format!("{}_", self.left_matcher.pattern))
            } else {
                ("right", format!("_{}", self.right_matcher.pattern))
            }
        } else if self.right_matcher.status {
            ("right", format!("_{}", self.right_matcher.pattern))
        } else if self.left_matcher.status {
            ("left", format!("{}_", self.left_matcher.pattern))
        } else {
            ("unknown", String::from("unknown"))
        }
    }
}

/// Matcher structure
#[derive(Debug, Clone)]
pub struct Matcher {
    pattern: String,
    score: i32,
    pub ystart: usize,
    pub yend: usize,
    pub status: bool,
}

impl Matcher {
    /// Create new matcher
    pub fn new() -> Self {
        Self {
            pattern: String::from(""),
            score: 99,
            ystart: 0,
            yend: 0,
            status: false,
        }
    }
    
    /// Get match score
    pub fn get_score(&self) -> i32 {
        self.score
    }
}

/// Calculate start and end positions
fn calculate_start_end_positions(
    start: usize,
    end: usize,
    position_shift: usize,
    pattern_length: usize,
    text_length: usize,
    orientation: &'static str,
) -> (usize, usize) {
    let mut new_start = start;
    let mut new_end = end;
    
    match orientation {
        "left" => {
            new_start = end
                .checked_sub(pattern_length)
                .and_then(|x| x.checked_sub(position_shift))
                .unwrap_or(0);
            new_end = min(text_length, new_end + position_shift);
        }
        "right" => {
            new_end = min(text_length, new_start + pattern_length + position_shift);
            new_start = start.checked_sub(position_shift).unwrap_or(0);
        }
        _ => {}
    }
    
    (new_start, new_end)
}

/// Find matcher
fn find_matcher(
    raw_start: usize,
    raw_end: usize,
    pattern_database: &HashMap<String, String>,
    search_pattern: &mut SearchPattern,
    use_position_mutation: bool,
    position_shift: usize,
    orientation: &'static str,
) -> Matcher {
    let mut matcher = Matcher::new();
    
    for (key, value) in pattern_database.iter() {
        let pattern = value.as_bytes().to_vec();
        let (start_pos, end_pos) = if use_position_mutation {
            calculate_start_end_positions(
                raw_start,
                raw_end,
                position_shift,
                pattern.len(),
                search_pattern.raw_text_len,
                orientation,
            )
        } else {
            (raw_start, raw_end)
        };
        
        search_pattern.update(start_pos, end_pos, pattern);
        
        if let Some(result) = myers_best(search_pattern) {
            if result.0 < matcher.score {
                matcher.pattern = key.to_string();
                matcher.score = result.0;
                matcher.ystart = result.1;
                matcher.yend = result.2;
                matcher.status = true;
            }
        }
    }
    
    matcher
}

/// Execute sequence splitting
fn perform_sequence_splitting(
    record: &Record, 
    read_chunk: &ReadChunk, 
    pattern_argument: &PatternArgument
) -> SplitType {
    let pattern_database = &pattern_argument.pattern_database;
    let mut search_pattern = SearchPattern::new(
        record.seq().to_vec(), 
        pattern_argument.pattern_error_rate.0
    );
    
    // Search left pattern
    let left_matcher = find_matcher(
        0,
        read_chunk.left_bound,
        &pattern_database.forward_patterns,
        &mut search_pattern,
        read_chunk.use_position_mutation,
        pattern_argument.position_shift,
        "left",
    );
    
    // Search right pattern
    search_pattern.dist_ratio = pattern_argument.pattern_error_rate.1;
    let right_matcher = find_matcher(
        read_chunk.right_bound,
        record.seq().len(),
        &pattern_database.reverse_patterns,
        &mut search_pattern,
        read_chunk.use_position_mutation,
        pattern_argument.position_shift,
        "right",
    );
    
    let mut split_type = SplitType::new(left_matcher, right_matcher);
    split_type.annotate_pattern_type(
        &pattern_database.pattern_types, 
        pattern_argument.max_distance as i32
    );
    
    split_type
}

/// Execute sequence splitting向量
pub fn perform_sequence_splitting_vector(
    read_info: &ReadInfo, 
    pattern_config: &PatternConfiguration
) -> Vec<SplitType> {
    let mut split_types = Vec::new();
    let mut read_chunk = ReadChunk::new(pattern_config, read_info);
    
    for pattern_argument in &pattern_config.pattern_arguments {
        let split_type = perform_sequence_splitting(&read_info.record, &read_chunk, pattern_argument);
        
        if pattern_argument.use_position_info
            && split_type.left_matcher.status
            && split_type.right_matcher.status
        {
            read_chunk.left_bound = split_type.left_matcher.ystart;
            read_chunk.right_bound = split_type.right_matcher.yend;
            read_chunk.use_position_mutation = true;
        } else {
            read_chunk = ReadChunk::new(pattern_config, read_info);
        }
        
        split_types.push(split_type);
    }
    
    split_types
}

/// Detect fusion sequence
fn detect_fusion_sequence(read_info: &ReadInfo, pattern_config: &PatternConfiguration) -> bool {
    let (middle_start, middle_end) = read_info.sequence_window;
    
    if middle_end <= middle_start {
        return false;
    }
    
    let fusion_database = &pattern_config.fusion_database.fusion_patterns;
    let mut search_pattern = SearchPattern::new(
        read_info.record.seq().to_vec(), 
        pattern_config.fusion_error_rate
    );

    // Search patterns in middle section
    let middle_matcher = find_matcher(
        middle_start,
        middle_end,
        fusion_database,
        &mut search_pattern,
        false,
        0,
        "middle",
    );

    middle_matcher.status
}

/// Create splitter receiver
pub fn create_splitter_receiver(
    read_receiver: Receiver<ReadInfo>,
    pattern_config: &PatternConfiguration,
    thread_count: usize,
) -> Receiver<ReadInfo> {
    let (sender, receiver) = flume::unbounded();
    
    for _thread_id in 0..thread_count {
        let start_time = Instant::now();
        let read_receiver = read_receiver.clone();
        let sender = sender.clone();
        let pattern_config = pattern_config.clone();
        
        thread::spawn(move || {
            let mut _processed_count = 0;
            
            for mut read_info in read_receiver.iter() {
                read_info.split_types = perform_sequence_splitting_vector(&read_info, &pattern_config);
                
                // Update sequence information
                read_info.update(
                    &pattern_config.pattern_match_types,
                    &pattern_config.write_type,
                    pattern_config.trim_mode,
                    pattern_config.min_length,
                    &pattern_config.id_separator,
                );
                
                // Detect fusion sequence
                if !pattern_config.fusion_database.is_empty() 
                    && detect_fusion_sequence(&read_info, &pattern_config) 
                {
                    read_info.sequence_type = "fusion".into();
                    read_info.should_write_to_fastq = false;
                }
                
                sender.send(read_info).expect("Failed to send sequence information");
                _processed_count += 1;
            }
            
            let _elapsed_time = start_time.elapsed();
            // Thread processing complete, no log output to avoid interference
        });
    }
    
    receiver
}

/// Create controlled splitter receiver with thread pool management
pub fn create_splitter_receiver_controlled(
    read_receiver: Receiver<ReadInfo>,
    pattern_config: &PatternConfiguration,
    thread_count: usize,
    thread_pool: &mut ThreadPoolManager,
) -> Receiver<ReadInfo> {
    let (sender, receiver) = flume::unbounded();
    
    // 分配线程资源
    let allocated_threads = thread_pool.allocate_threads(thread_count);
    
    for _thread_id in 0..allocated_threads {
        let start_time = Instant::now();
        let read_receiver = read_receiver.clone();
        let sender = sender.clone();
        let pattern_config = pattern_config.clone();
        
        // 使用受控的线程创建
        if let Some(_handle) = thread_pool.spawn_controlled_thread(move || {
            let mut _processed_count = 0;
            
            for mut read_info in read_receiver.iter() {
                read_info.split_types = perform_sequence_splitting_vector(&read_info, &pattern_config);
                
                // Update sequence information
                read_info.update(
                    &pattern_config.pattern_match_types,
                    &pattern_config.write_type,
                    pattern_config.trim_mode,
                    pattern_config.min_length,
                    &pattern_config.id_separator,
                );
                
                // Detect fusion sequence
                if !pattern_config.fusion_database.is_empty() 
                    && detect_fusion_sequence(&read_info, &pattern_config) 
                {
                    read_info.sequence_type = "fusion".into();
                    read_info.should_write_to_fastq = false;
                }
                
                sender.send(read_info).expect("Failed to send sequence information");
                _processed_count += 1;
            }
            
            let _elapsed_time = start_time.elapsed();
            // Thread processing complete, no log output to avoid interference
        }) {
            // 线程创建成功，继续处理
        } else {
            // 线程创建失败，释放资源
            thread_pool.release_threads(1);
        }
    }
    
    receiver
}