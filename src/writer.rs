use std::collections::HashMap;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::Semaphore;
use async_compression::tokio::write::GzipEncoder;
use log::info;
use std::io::Result;
use std::path::Path;
use tokio::fs::create_dir_all;
use crate::fastq::ReadInfo;
use tokio::io::BufWriter;
use tokio::task::JoinHandle;
use flume::{Receiver, Sender, unbounded};

/// File write manager
pub struct FileWriterManager {
    /// Writer mapping
    writers: HashMap<String, Sender<ReadInfo>>,
    /// Output directory
    output_directory: String,
    /// Logger
    pub logger: Vec<String>,
    /// Task handles
    task_handles: Vec<JoinHandle<()>>,
    /// Semaphore to limit concurrent writing tasks (max 4)
    write_semaphore: Arc<Semaphore>,
}

impl FileWriterManager {

    /// Create file write manager
    pub fn new(output_directory: String) -> Self {
        info!("Creating writer manager with 4 concurrent write tasks limit...");
        Self {
            writers: HashMap::new(),
            output_directory,
            logger: Vec::new(),
            task_handles: Vec::new(),
            write_semaphore: Arc::new(Semaphore::new(4)),
        }
    }

    /// Write sequence information (non-blocking)
    pub fn write(&mut self, read_info: ReadInfo) -> Result<()> {
        if !read_info.should_write_to_fastq {
            return Ok(());
        }
        
        let output_filename = read_info.output_filename.clone();
        
        if !self.writers.contains_key(&output_filename) {
            let (tx, rx) = unbounded();
            let file_path = Path::new(&self.output_directory)
                .join(format!("{}.fq.gz", output_filename));
            
            // Start writing task that will create file and handle all I/O
            self.start_writing_task(file_path, rx);
            self.writers.insert(output_filename.clone(), tx);
        }
        
        self.writers.get(&output_filename).unwrap()
            .send(read_info)
            .expect("Failed to send read info to writer");
        
        Ok(())
    }

    fn start_writing_task(&mut self, file_path: std::path::PathBuf, rx: Receiver<ReadInfo>) {
        let semaphore = Arc::clone(&self.write_semaphore);
        let handle = tokio::spawn(async move {
            // Create directory and file asynchronously in the task
            let file_directory = file_path.parent().unwrap();
            create_dir_all(&file_directory).await
                .expect("Failed to create output directory");
            
            let file = File::create(&file_path).await
                .expect("Failed to create output file");
            
            let encoder = GzipEncoder::new(file);
            let mut writer = BufWriter::with_capacity(40_000_000, encoder);
            
            let mut batch_count = 0;
            const BATCH_SIZE: usize = 1000; // Process in batches of 1000 records
            
            // Process each read_info immediately as it arrives
            for read_info in rx.iter() {
                if let Some(output_record) = read_info.get_output_record() {
                    let id = output_record.id();
                    let seq = std::str::from_utf8(output_record.seq())
                        .expect("Not a valid UTF-8 sequence");
                    let qual = std::str::from_utf8(output_record.qual())
                        .expect("Not a valid UTF-8 sequence");
                    let record_str = format!("@{}\n{}\n+\n{}\n", id, seq, qual);
                    
                    writer.write_all(record_str.as_bytes()).await.unwrap();
                    batch_count += 1;
                    
                    // Periodically flush and yield control to limit CPU usage
                    if batch_count >= BATCH_SIZE {
                        let _permit = semaphore.acquire().await.expect("Semaphore closed");
                        writer.flush().await.expect("Failed to flush writer buffer");
                        drop(_permit); // Release permit immediately after flush
                        batch_count = 0;
                    }
                }
            }
            
            // Final flush
            let _permit = semaphore.acquire().await.expect("Semaphore closed");
            writer.flush().await.expect("Failed to flush writer buffer");
            
            // Finish gzip encoder
            let mut encoder = writer.into_inner();
            encoder.shutdown().await.expect("Failed to finish gzip encoding");
        });
        
        self.task_handles.push(handle);
    }

    /// Write log file
    pub async fn write_log_file(&self, output_directory: &str) -> Result<()> {
        let directory_path = Path::new(output_directory);
        create_dir_all(&directory_path).await?;
        
        info!("Writing logs to reads_log.gz");
        let file_path = directory_path.join("reads_log.gz");
        let file = File::create(file_path).await?;
        let mut encoder = GzipEncoder::new(file);
        
        for line in &self.logger {
            encoder.write_all(line.as_ref()).await?;
            encoder.write_all(b"\n").await?;
        }
        
        encoder.shutdown().await?;
        Ok(())
    }
    
    /// Complete writing and wait for all tasks to finish
    pub async fn finish(&mut self) {
        info!("Writing FASTQ files, this may take some time...");
        
        // Drop all senders to close channels
        self.writers.clear();
        
        // Wait for all writing tasks to complete
        for handle in self.task_handles.drain(..) {
            handle.await.expect("Writing task panicked");
        }
        
        info!("All writing tasks completed successfully");
    }
    
}
