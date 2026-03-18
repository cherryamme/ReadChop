use std::collections::{HashMap, VecDeque};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use flume::{bounded, Receiver, Sender};
use log::info;
use crate::fastq::ReadInfo;
use flate2::write::GzEncoder;
use flate2::Compression;

/// Channel capacity for writer (controls memory usage by limiting buffer size)
/// When channel is full, send will block, providing backpressure
const CHANNEL_CAPACITY: usize = 1_000;

/// Buffer size for output (8MB)
const BUFFER_SIZE: usize = 10 * 1024;

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
    pub fn new(output_directory: String, writer_threads: usize, enable_logger: bool, uncompress: bool) -> Self {
        let compress = !uncompress;
        let compression_status = if compress { "enabled (gz)" } else { "disabled (fastq)" };
        info!("Creating writer manager with {} writer threads, compression: {}...", writer_threads, compression_status);
        
        // Create a separate channel for each worker thread
        let mut thread_senders = Vec::new();
        let mut thread_handles = Vec::new();
        
        // Create a global directory creation lock to prevent cluster filesystem concurrent race conditions
        let dir_lock = Arc::new(Mutex::new(()));
        
        for _ in 0..writer_threads {
            let (sender, receiver) = bounded(CHANNEL_CAPACITY);
            thread_senders.push(sender);
            
            let output_dir = output_directory.clone();
            let compress_flag = compress;
            let dir_lock_clone = Arc::clone(&dir_lock);
            
            let handle = thread::spawn(move || {
                Self::writer_worker(receiver, output_dir, compress_flag, dir_lock_clone);
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
        receiver: Receiver<ReadInfo>,
        output_directory: String,
        compress: bool,
        dir_lock: Arc<Mutex<()>>,
    ) {
        // HashMap to store file writers for each tag (output_filename)
        // Use enum to handle both compressed and uncompressed writers
        enum Writer {
            Compressed(BufWriter<GzEncoder<File>>),
            Uncompressed(BufWriter<File>),
        }
        
        impl Writer {
            fn write_fmt(&mut self, fmt: std::fmt::Arguments<'_>) -> std::io::Result<()> {
                match self {
                    Writer::Compressed(w) => w.write_fmt(fmt),
                    Writer::Uncompressed(w) => w.write_fmt(fmt),
                }
            }
        }
        
        let mut writers: HashMap<String, Writer> = HashMap::new();
        
        // 用于记录文件打开顺序的队列（FIFO）
        let mut open_queue: VecDeque<String> = VecDeque::new();
        
        // 每个线程最多同时保持 50 个打开的文件
        // 10个线程总共 500 个，对集群毫无压力
        const MAX_OPEN_FILES: usize = 50;
        
        // Process each ReadInfo from the channel
        for mut read_info in receiver.iter() {
            // Skip if not should write
            if !read_info.should_write_to_fastq {
                read_info.clear_large_data();
                continue;
            }
            
            let tag = read_info.output_filename.clone();
            
            // Get or create writer for this tag
            if !writers.contains_key(&tag) {
                
                // 【核心新增：如果达到上限，踢出（关闭）最老的文件】
                if writers.len() >= MAX_OPEN_FILES {
                    if let Some(oldest_tag) = open_queue.pop_front() {
                        if let Some(writer) = writers.remove(&oldest_tag) {
                            match writer {
                                Writer::Compressed(mut w) => {
                                    w.flush().unwrap();
                                    w.into_inner().unwrap().finish().unwrap();
                                },
                                Writer::Uncompressed(mut w) => {
                                    w.flush().unwrap();
                                }
                            }
                        }
                    }
                }

                let file_extension = if compress { ".fq.gz" } else { ".fq" };
                let mut file_path = std::path::PathBuf::from(&output_directory);
                let mut filename = String::with_capacity(tag.len() + file_extension.len());
                filename.push_str(&tag);
                filename.push_str(file_extension);
                file_path.push(&filename);
                
                // Create directory with thread-safe lock
                if let Some(parent) = file_path.parent() {
                    // First check without lock for performance optimization
                    if !parent.exists() {
                        let _lock = dir_lock.lock().unwrap();
                        // Check again after acquiring lock to prevent other threads from just creating it
                        if !parent.exists() {
                            std::fs::create_dir_all(parent)
                                .unwrap_or_else(|_| panic!("Failed to create output directory: {:?}", parent));
                            // Give cluster MDS some time to sync permission information
                            std::thread::sleep(std::time::Duration::from_millis(20));
                        }
                    }
                }
                
                // Open file with enhanced retry mechanism for cluster filesystems
                // 使用 append(true) 而非 truncate(true)，因为文件可能会被关闭后再次打开写入
                let mut attempts = 0;
                let file = loop {
                    match OpenOptions::new()
                        .write(true)
                        .create(true)
                        .append(true)  // 使用追加模式，允许断点续写
                        .open(&file_path) 
                    {
                        Ok(f) => break f,
                        Err(e) => {
                            attempts += 1;
                            if attempts >= 5 {
                                panic!("Failed to open file {:?} after 5 attempts: {}", file_path, e);
                            }
                            // Gradual delay to wait for cluster to release ghost locks
                            std::thread::sleep(std::time::Duration::from_millis(100 * attempts as u64));
                        }
                    }
                };
                // Create buffered writer (compressed or uncompressed)
                let writer = if compress {
                    let encoder = GzEncoder::new(file, Compression::new(1));
                    Writer::Compressed(BufWriter::with_capacity(BUFFER_SIZE, encoder))
                } else {
                    Writer::Uncompressed(BufWriter::with_capacity(BUFFER_SIZE, file))
                };
                
                writers.insert(tag.clone(), writer);
                open_queue.push_back(tag.clone()); // 记录到队列末尾
            }
            
            // Write the record
            if let Some(output_record) = read_info.get_output_record() {
                let id = output_record.id();
                let seq = std::str::from_utf8(output_record.seq())
                    .expect("Not a valid UTF-8 sequence");
                let qual = std::str::from_utf8(output_record.qual())
                    .expect("Not a valid UTF-8 sequence");
                
                let writer = writers.get_mut(&tag).unwrap();
                write!(writer, "@{}\n{}\n+\n{}\n", id, seq, qual)
                    .expect("Failed to write record");
            }
            
            // Clear large data immediately after writing to free memory
            read_info.clear_large_data();
        }
        
        // Final flush all writers
        for (tag, writer) in writers {
            match writer {
                Writer::Compressed(mut w) => {
                    w.flush().unwrap_or_else(|_| panic!("Failed to flush BufWriter for tag: {}", tag));
                    let encoder = w.into_inner()
                        .unwrap_or_else(|_| panic!("Failed to get encoder for tag: {}", tag));
                    encoder.finish()
                        .unwrap_or_else(|_| panic!("Failed to finish compression for tag: {}", tag));
                },
                Writer::Uncompressed(mut w) => {
                    w.flush().unwrap_or_else(|_| panic!("Failed to flush writer for tag: {}", tag));
                },
            }
        }
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
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        read_info.output_filename.hash(&mut hasher);
        let assigned_thread = (hasher.finish() as usize) % self.num_threads;
        
        // Send to the assigned thread's channel - this will block if channel is full (backpressure)
        self.thread_senders[assigned_thread]
            .send(read_info)
            .expect("Failed to send read info to writer channel");
        
        Ok(())
    }
    
    /// Expand writer concurrency after splitter completes
    /// With the channel-based design, work is distributed automatically
    pub fn expand_concurrency(&self, _additional_threads: usize) {
        // No-op: work is already distributed via channels
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
        let mut encoder = GzEncoder::new(file, Compression::new(1));
        
        for line in &self.logger {
            encoder.write_all(line.as_bytes())?;
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
        
        info!("All {} writer threads completed successfully", self.num_threads);
    }
}

