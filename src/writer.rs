use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::thread;
use flume::{bounded, Receiver, Sender};
use log::info;
use crate::fastq::ReadInfo;
use flate2::write::GzEncoder;
use flate2::Compression;

/// Channel capacity for writer (controls memory usage by limiting buffer size)
/// When channel is full, send will block, providing backpressure
const CHANNEL_CAPACITY: usize = 1_000;

/// Batch size for periodic flushing
const BATCH_SIZE: usize = 1000;

/// Buffer size for output (8MB)
const BUFFER_SIZE: usize = 4 * 1024 * 1024;

/// File write manager with tag-based sharding
pub struct FileWriterManager {
    /// Channel senders for each worker thread (one per thread)
    thread_senders: Vec<Sender<ReadInfo>>,
    /// Number of writer threads
    num_threads: usize,
    /// Logger
    pub logger: Vec<String>,
    /// Whether logger is enabled
    enable_logger: bool,
    /// Thread handles for writer threads
    thread_handles: Vec<thread::JoinHandle<()>>,
}

impl FileWriterManager {
    /// Create file write manager with tag-based sharding
    pub fn new(output_directory: String, writer_threads: usize, enable_logger: bool, compress: bool) -> Self {
        let compression_status = if compress { "enabled (gz)" } else { "disabled (fastq)" };
        info!("Creating writer manager with {} writer threads, compression: {}...", writer_threads, compression_status);
        
        // Create a separate channel for each worker thread
        let mut thread_senders = Vec::new();
        let mut thread_handles = Vec::new();
        
        for thread_id in 0..writer_threads {
            let (sender, receiver) = bounded(CHANNEL_CAPACITY);
            thread_senders.push(sender);
            
            let output_dir = output_directory.clone();
            let compress_flag = compress;
            
            let handle = thread::spawn(move || {
                Self::writer_worker(thread_id, receiver, output_dir, compress_flag);
            });
            
            thread_handles.push(handle);
        }
        
        Self {
            thread_senders,
            num_threads: writer_threads,
            logger: Vec::new(),
            enable_logger,
            thread_handles,
        }
    }
    
    /// Writer worker thread - processes ReadInfo from its dedicated channel
    /// Each thread only receives data assigned to it based on tag hash
    fn writer_worker(
        thread_id: usize,
        receiver: Receiver<ReadInfo>,
        output_directory: String,
        compress: bool,
    ) {
        // HashMap to store file writers for each tag (output_filename)
        // Use enum to handle both compressed and uncompressed writers
        enum Writer {
            Compressed(BufWriter<GzEncoder<File>>),
            Uncompressed(BufWriter<File>),
        }
        
        impl Writer {
            fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
                match self {
                    Writer::Compressed(w) => w.write_all(buf),
                    Writer::Uncompressed(w) => w.write_all(buf),
                }
            }
            
            fn flush(&mut self) -> std::io::Result<()> {
                match self {
                    Writer::Compressed(w) => w.flush(),
                    Writer::Uncompressed(w) => w.flush(),
                }
            }
        }
        
        let mut writers: HashMap<String, Writer> = HashMap::new();
        let mut batch_counters: HashMap<String, usize> = HashMap::new();
        
        // Process each ReadInfo from the channel
        // All data received here is already assigned to this thread
        for mut read_info in receiver.iter() {
            // Skip if not should write
            if !read_info.should_write_to_fastq {
                read_info.clear_large_data();
                continue;
            }
            
            let tag = read_info.output_filename.clone();
            
            // Get or create writer for this tag
            let is_new_writer = !writers.contains_key(&tag);
            if is_new_writer {
                let file_extension = if compress { ".fq.gz" } else { ".fq" };
                let file_path = Path::new(&output_directory)
                    .join(format!("{}{}", tag, file_extension));
                
                // Create directory
                if let Some(parent) = file_path.parent() {
                    std::fs::create_dir_all(parent)
                        .expect(&format!("Failed to create output directory: {:?}", parent));
                }
                
                // Open file - use create mode (will overwrite if exists, but should only be created once per thread)
                let file = File::create(&file_path)
                    .expect(&format!("Failed to create output file: {:?}", file_path));
                
                // Create buffered writer (compressed or uncompressed)
                let writer = if compress {
                    let encoder = GzEncoder::new(file, Compression::new(1));
                    Writer::Compressed(BufWriter::with_capacity(BUFFER_SIZE, encoder))
                } else {
                    Writer::Uncompressed(BufWriter::with_capacity(BUFFER_SIZE, file))
                };
                
                writers.insert(tag.clone(), writer);
                batch_counters.insert(tag.clone(), 0);
                info!("Thread {}: Created new writer for tag: {}", thread_id, tag);
            }
            
            // Write the record
            if let Some(output_record) = read_info.get_output_record() {
                let id = output_record.id();
                let seq = std::str::from_utf8(output_record.seq())
                    .expect("Not a valid UTF-8 sequence");
                let qual = std::str::from_utf8(output_record.qual())
                    .expect("Not a valid UTF-8 sequence");
                let record_str = format!("@{}\n{}\n+\n{}\n", id, seq, qual);
                
                let writer = writers.get_mut(&tag).unwrap();
                writer.write_all(record_str.as_bytes())
                    .expect("Failed to write record");
                
                // Update batch counter
                let counter = batch_counters.get_mut(&tag).unwrap();
                *counter += 1;
                
            }
            
            // Clear large data immediately after writing to free memory
            read_info.clear_large_data();
        }
        
        // Final flush all writers
        for (tag, writer) in writers {
            // For compressed writers, finish the encoder
            match writer {
                Writer::Compressed(mut w) => {
                    // Flush the BufWriter first
                    w.flush().expect(&format!("Failed to flush BufWriter for tag: {}", tag));
                    // Get the inner GzEncoder and finish it
                    let encoder = w.into_inner()
                        .expect(&format!("Failed to get encoder for tag: {}", tag));
                    encoder.finish()
                        .expect(&format!("Failed to finish compression for tag: {}", tag));
                },
                Writer::Uncompressed(mut w) => {
                    w.flush().expect(&format!("Failed to flush writer for tag: {}", tag));
                },
            }
        }
        
        info!("Writer thread {} completed", thread_id);
    }
    
    /// Log record (only if logger is enabled)
    pub fn log(&mut self, log_line: String) {
        if self.enable_logger {
            self.logger.push(log_line);
        }
    }

    /// Write sequence information (blocking if channel is full to control memory usage)
    /// This provides backpressure: when channel is full, send blocks
    /// Routes data to the correct thread based on tag hash
    pub fn write(&mut self, read_info: ReadInfo) -> std::io::Result<()> {
        // Calculate which thread should handle this tag
        let tag = &read_info.output_filename;
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        tag.hash(&mut hasher);
        let hash_value = hasher.finish();
        let assigned_thread = (hash_value as usize) % self.num_threads;
        
        // Send to the assigned thread's channel - this will block if channel is full (backpressure)
        if let Some(sender) = self.thread_senders.get(assigned_thread) {
            sender.send(read_info)
                .expect("Failed to send read info to writer channel");
        }
        
        Ok(())
    }
    
    /// Expand writer concurrency after splitter completes
    /// This method is kept for API compatibility but doesn't need to do anything
    pub fn expand_concurrency(&self, _additional_threads: usize) {
        // Note: With the channel-based design, work is distributed automatically
        // This method is kept for API compatibility but doesn't need to do anything
    }

    /// Write log file (only if logger is enabled)
    pub fn write_log_file(&self, output_directory: &str) -> std::io::Result<()> {
        if !self.enable_logger {
            return Ok(());
        }
        
        let directory_path = Path::new(output_directory);
        std::fs::create_dir_all(&directory_path)?;
        
        info!("Writing logs to reads_log.gz");
        let file_path = directory_path.join("reads_log.gz");
        let file = File::create(file_path)?;
        
        // Use flate2 for gzip compression
        use flate2::write::GzEncoder;
        use flate2::Compression;
        let mut encoder = GzEncoder::new(file, Compression::new(1));
        
        for line in &self.logger {
            encoder.write_all(line.as_ref())?;
            encoder.write_all(b"\n")?;
        }
        
        encoder.finish()?;
        Ok(())
    }
    
    /// Complete writing and wait for all threads to finish
    pub fn finish(&mut self) {
        info!("Writing FASTQ files, this may take some time...");
        
        // Drop all senders to close channels (signals workers to finish)
        self.thread_senders.clear();
        
        // Wait for all writing threads to complete
        for handle in self.thread_handles.drain(..) {
            handle.join().expect("Writing thread panicked");
        }
        
        info!("All writing tasks completed successfully");
    }
}

