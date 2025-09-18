use crate::splitter::SplitType;
use bio::io::fastq::{Reader, Record};
use flate2::read::MultiGzDecoder;
use flume::{unbounded, Sender, Receiver};
use log::info;
use std::ffi::OsStr;
use std::{
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
};
use std::time::Instant;
use std::collections::HashSet;

/// Buffer size constant for I/O performance optimization
const BUFFER_SIZE: usize = 10 * 1024 * 1024;

/// Check if file is gzip compressed format
fn is_gzip_file(path: &PathBuf) -> bool {
    match path.extension().and_then(OsStr::to_str) {
        Some(ext) => ext == "gz",
        None => false,
    }
}

/// Create FASTQ reader, return receiver
pub fn create_reader(files: Vec<String>) -> Receiver<ReadInfo> {
    let (sender, receiver) = unbounded();
    
    std::thread::spawn(move || {
        let start_time = Instant::now();
        
        if files.is_empty() {
            info!("No input files specified, reading from standard input...");
            let stdin_handle = std::io::stdin();
            process_file(stdin_handle, &sender, None);
        } else {
            for file_path in files {
                let path = PathBuf::from(&file_path);
                if path.exists() {
                    let file_handle = File::open(&path)
                        .expect(&format!("Unable to open input file: {}", path.display()));
                    process_file(file_handle, &sender, Some(path));
                } else {
                    panic!("File does not exist: {}", path.display());
                }
            }
        }

        let elapsed_time = start_time.elapsed();
        info!("Reading sequence data completed! Time taken: {:.4?}", elapsed_time);
    });
    
    receiver
}

/// Process single file
fn process_file<R: Read + 'static>(
    file_handle: R, 
    sender: &Sender<ReadInfo>, 
    file_path: Option<PathBuf>
) {
    let buffered_reader = BufReader::with_capacity(BUFFER_SIZE, file_handle);
    let decoder_handle = create_decoder(buffered_reader, file_path);
    let fastq_reader = Reader::new(decoder_handle);
    
    for record_result in fastq_reader.records() {
        let record = record_result.expect("Failed to read FASTQ record");
        let read_info = ReadInfo::new(record);
        sender.send(read_info).expect("Failed to send sequence information");
    }
}

/// Create appropriate decoder
fn create_decoder<R: Read + 'static>(
    buffered_reader: BufReader<R>, 
    file_path: Option<PathBuf>
) -> Box<dyn Read> {
    match file_path {
        Some(path) if is_gzip_file(&path) => {
            info!("Loading gzip compressed file: {:?}", path);
            Box::new(MultiGzDecoder::new(buffered_reader)) as Box<dyn Read>
        }
        Some(path) => {
            info!("Loading FASTQ file: {:?}", path);
            Box::new(buffered_reader) as Box<dyn Read>
        }
        None => Box::new(buffered_reader) as Box<dyn Read>,
    }
}

/// Sequence information structure
#[derive(Debug, Clone)]
pub struct ReadInfo {
    /// Original FASTQ record
    pub record: Record,
    /// Split type vector
    pub split_types: Vec<SplitType>,
    /// Output filename
    pub output_filename: String,
    /// Strand direction
    pub strand_orientation: String,
    /// Sequence type
    pub sequence_type: String,
    /// Match type list
    pub match_types: Vec<String>,
    /// Match name list
    pub match_names: Vec<String>,
    /// Record ID
    pub record_id: String,
    /// Whether to write FASTQ file
    pub should_write_to_fastq: bool,
    /// Output record
    pub output_record: Record,
    /// Sequence length
    pub sequence_length: usize,
    /// Sequence window position
    pub sequence_window: (usize, usize),
}

impl ReadInfo {
    /// Create new sequence information
    pub fn new(record: Record) -> Self {
        let sequence_length = record.seq().len();
        Self {
            record: record.clone(),
            split_types: Vec::new(),
            output_filename: String::new(),
            strand_orientation: String::from("unknown"),
            sequence_type: String::from("valid"),
            match_types: Vec::new(),
            match_names: Vec::new(),
            record_id: String::new(),
            should_write_to_fastq: false,
            output_record: Record::new(),
            sequence_length,
            sequence_window: (0, sequence_length),
        }
    }
    
    /// Update sequence information
    pub fn update(
        &mut self, 
        pattern_match_types: &[String], 
        write_type: &str, 
        trim_mode: usize, 
        min_length: usize, 
        id_separator: &str
    ) {
        self.update_match_names(pattern_match_types);
        self.update_output_filename(write_type, id_separator);
        self.update_sequence_type(min_length, trim_mode);
        self.update_sequence_window();
        self.update_write_decision(trim_mode, id_separator);
    }
    
    /// Update match names
    fn update_match_names(&mut self, pattern_match_types: &[String]) {
        let mut strand_values = Vec::new();
        
        for (index, split_type) in self.split_types.iter().enumerate() {
            match pattern_match_types.get(index) {
                Some(match_type) if match_type >= &String::from(split_type.pattern_match) => {
                    self.match_types.push(split_type.pattern_type.clone());
                    self.match_names.push(split_type.pattern_name.clone());
                }
                _ => {
                    self.match_types.push(String::from("unknown"));
                    self.match_names.push(String::from("unknown"));
                    self.sequence_type = "unknown".to_string();
                }
            }
            strand_values.push(split_type.pattern_strand.clone());
        }
        
        // Ensure at least 3 elements
        while self.match_names.len() < 3 {
            self.match_names.push(String::from("default"));
        }
        while self.match_types.len() < 3 {
            self.match_types.push(String::from("default"));
        }
        
        // Determine strand direction
        let unique_strands: HashSet<_> = strand_values.drain(..).collect();
        if unique_strands.len() == 1 && !unique_strands.contains("unknown") {
            self.strand_orientation = unique_strands.into_iter().next().unwrap();
        }
    }
    
    /// Update output filename
    fn update_output_filename(&mut self, write_type: &str, id_separator: &str) {
        if write_type == "type" {
            let mut reversed_types = self.match_types.clone();
            reversed_types.reverse();
            self.output_filename = reversed_types.join("/");
            self.record_id = self.match_types.join(id_separator);
        } else {
            let mut reversed_names = self.match_names.clone();
            reversed_names.reverse();
            self.output_filename = reversed_names.join("/");
            self.record_id = self.match_names.join(id_separator);
        }
    }
    
    /// Update sequence window
    pub fn update_sequence_window(&mut self) {
        if let Some(first_split) = self.split_types.first() {
            if first_split.left_matcher.status {
                self.sequence_window.0 = first_split.left_matcher.yend;
            }
            if first_split.right_matcher.status {
                self.sequence_window.1 = first_split.right_matcher.ystart;
            }
        }
    }
    
    /// Update sequence type
    fn update_sequence_type(&mut self, min_length: usize, trim_mode: usize) {
        if self.sequence_length <= min_length {
            self.sequence_type = "filtered".to_string();
        }
        
        let (cut_left, cut_right) = self.calculate_trim_positions(trim_mode);
        
        if cut_left > cut_right {
            self.sequence_type = "unknown".to_string();
            self.should_write_to_fastq = false;
        }
    }
    
    /// Calculate trim positions
    fn calculate_trim_positions(&self, trim_mode: usize) -> (usize, usize) {
        if trim_mode == 0 {
            if let Some(first_split) = self.split_types.first() {
                (
                    first_split.left_matcher.yend,
                    first_split.right_matcher.ystart,
                )
            } else {
                (0, self.sequence_length)
            }
        } else if trim_mode <= self.split_types.len() {
            let split = &self.split_types[trim_mode - 1];
            (split.left_matcher.ystart, split.right_matcher.yend)
        } else {
            (0, self.sequence_length)
        }
    }
    
    /// Update write decision
    fn update_write_decision(&mut self, trim_mode: usize, id_separator: &str) {
        if self.sequence_type == "valid" {
            self.should_write_to_fastq = true;
            let (cut_left, cut_right) = self.calculate_trim_positions(trim_mode);
            let final_cut_right = if cut_right == 0 { self.sequence_length } else { cut_right };
            
            self.output_record = Record::with_attrs(
                &format!("{}{}{}{}{}", 
                    self.record.id(), 
                    id_separator, 
                    self.strand_orientation, 
                    id_separator, 
                    self.record_id
                ),
                None,
                &self.record.seq()[cut_left..final_cut_right],
                &self.record.qual()[cut_left..final_cut_right],
            );
        }
    }
    
    /// Convert to TSV format string
    pub fn to_tsv(&self) -> String {
        let mut tsv_line = format!(
            "{}\t{}\t{}", 
            self.record.id(), 
            self.sequence_length, 
            self.sequence_type
        );
        
        for split_type in &self.split_types {
            tsv_line.push_str(&format!("\t{}", split_type.to_info()));
        }
        
        tsv_line
    }
    
}