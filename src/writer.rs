use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use flate2::write::GzEncoder;
use flate2::Compression;
use log::info;
use std::io::Result;
use std::path::Path;
use std::fs::create_dir_all;
use crate::fastq::ReadInfo;
use crate::thread_pool::ThreadPoolManager;
use std::io::BufWriter;
use std::thread;
use flume::{Receiver, Sender, unbounded};

/// File write manager
pub struct FileWriterManager {
    /// Writer mapping
    writers: HashMap<String, Sender<ReadInfo>>,
    /// Output directory
    output_directory: String,
    /// Logger
    pub logger: Vec<String>,
    /// Thread handles
    thread_handles: Vec<thread::JoinHandle<()>>,
}

impl FileWriterManager {
    /// Create new file write manager
    pub fn new(output_directory: String) -> Self {
        info!("Creating file writer manager, starting writing...");
        Self {
            writers: HashMap::new(),
            output_directory,
            logger: Vec::new(),
            thread_handles: Vec::new(),
        }
    }

    /// Create controlled file write manager with thread pool management
    pub fn new_controlled(
        output_directory: String, 
        _max_writing_threads: usize,
        _thread_pool: &mut ThreadPoolManager
    ) -> Self {
        info!("Creating controlled file writer manager, max writing threads: {}", _max_writing_threads);
        Self {
            writers: HashMap::new(),
            output_directory,
            logger: Vec::new(),
            thread_handles: Vec::new(),
        }
    }

    /// Write sequence information
    pub fn write(&mut self, read_info: ReadInfo) -> Result<()> {
        if !read_info.should_write_to_fastq {
            return Ok(());
        }
        
        let output_filename = read_info.output_filename.clone();
        
        if !self.writers.contains_key(&output_filename) {
            self.create_writer_for_filename(&output_filename);
        }
        
        self.writers
            .get(&output_filename)
            .unwrap()
            .send(read_info)
            .expect("Failed to send sequence information to writer");
        
        Ok(())
    }

    /// Write sequence information with controlled thread management
    pub fn write_controlled(&mut self, read_info: ReadInfo, thread_pool: &mut ThreadPoolManager) -> Result<()> {
        if !read_info.should_write_to_fastq {
            return Ok(());
        }
        
        let output_filename = read_info.output_filename.clone();
        
        if !self.writers.contains_key(&output_filename) {
            self.create_writer_for_filename_controlled(&output_filename, thread_pool);
        }
        
        if let Some(sender) = self.writers.get(&output_filename) {
            sender.send(read_info)
                .expect("Failed to send sequence information to writer");
        }
        
        Ok(())
    }

    /// Create writer for filename
    fn create_writer_for_filename(&mut self, output_filename: &str) {
        let (sender, receiver) = unbounded();
        let file_path = Path::new(&self.output_directory)
            .join(format!("{}.fq.gz", output_filename));
        let file_directory = file_path.parent().unwrap();
        
        create_dir_all(&file_directory)
            .expect("Failed to create output directory");
        
        let file = File::create(&file_path)
            .expect("Failed to create output file");
        
        let encoder = GzEncoder::new(file, Compression::default());
        let writer = BufWriter::with_capacity(1_000_000, encoder);
        
        self.start_writing_thread(writer, receiver);
        self.writers.insert(output_filename.to_string(), sender);
    }

    /// Create controlled writer for filename with thread pool management
    fn create_writer_for_filename_controlled(&mut self, output_filename: &str, thread_pool: &mut ThreadPoolManager) {
        // 检查是否可以创建新的写入线程
        if !thread_pool.can_spawn_thread() {
            info!("无法创建新的写入线程，线程池已满");
            return;
        }

        let (sender, receiver) = unbounded();
        let file_path = Path::new(&self.output_directory)
            .join(format!("{}.fq.gz", output_filename));
        let file_directory = file_path.parent().unwrap();
        
        create_dir_all(&file_directory)
            .expect("Failed to create output directory");
        
        let file = File::create(&file_path)
            .expect("Failed to create output file");
        
        let encoder = GzEncoder::new(file, Compression::default());
        let writer = BufWriter::with_capacity(1_000_000, encoder);
        
        self.start_writing_thread_controlled(writer, receiver, thread_pool);
        self.writers.insert(output_filename.to_string(), sender);
    }

    /// Start write thread
    fn start_writing_thread(&mut self, mut writer: BufWriter<GzEncoder<File>>, receiver: Receiver<ReadInfo>) {
        let handle = thread::spawn(move || {
            for read_info in receiver.iter() {
                let record_id = read_info.output_record.id();
                let sequence = std::str::from_utf8(read_info.output_record.seq())
                    .expect("Sequence is not valid UTF-8");
                let quality = std::str::from_utf8(read_info.output_record.qual())
                    .expect("Quality scores are not valid UTF-8");
                
                let record_string = format!("@{}\n{}\n+\n{}\n", record_id, sequence, quality);
                write!(writer, "{}", record_string)
                    .expect("Failed to write sequence record");
            }
        });
        self.thread_handles.push(handle);
    }

    /// Start controlled write thread with thread pool management
    fn start_writing_thread_controlled(&mut self, mut writer: BufWriter<GzEncoder<File>>, receiver: Receiver<ReadInfo>, thread_pool: &mut ThreadPoolManager) {
        if let Some(handle) = thread_pool.spawn_controlled_thread(move || {
            for read_info in receiver.iter() {
                let record_id = read_info.output_record.id();
                let sequence = std::str::from_utf8(read_info.output_record.seq())
                    .expect("Sequence is not valid UTF-8");
                let quality = std::str::from_utf8(read_info.output_record.qual())
                    .expect("Quality scores are not valid UTF-8");
                
                let record_string = format!("@{}\n{}\n+\n{}\n", record_id, sequence, quality);
                write!(writer, "{}", record_string)
                    .expect("Failed to write sequence record");
            }
        }) {
            self.thread_handles.push(handle);
        } else {
            info!("无法创建受控的写入线程");
        }
    }

    /// Write log file
    pub fn write_log_file(&self, output_directory: &str) -> Result<()> {
        let directory_path = Path::new(output_directory);
        create_dir_all(&directory_path)?;
        
        info!("Writing logs to reads_log.gz");
        let file_path = directory_path.join("reads_log.gz");
        let file = File::create(file_path)?;
        let mut encoder = GzEncoder::new(file, Compression::default());
        
        for line in &self.logger {
            encoder.write_all(line.as_ref())?;
            encoder.write_all(b"\n")?;
        }
        
        encoder.finish()?;
        Ok(())
    }
    
    /// Complete writing and wait for all threads to finish
    pub fn finalize(&mut self) {
        info!("Writing FASTQ files, this may take some time...");
        
        // Clear writers, this will cause receivers to disconnect
        self.writers.clear();
        
        // Wait for all write threads to complete
        for handle in self.thread_handles.drain(..) {
            handle.join().expect("Writing thread panicked");
        }
    }
    
}