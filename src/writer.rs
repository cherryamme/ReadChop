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

    /// Create file write manager
    pub fn new(output_directory: String) -> Self {
        info!("Creating writer manager, start writing...");
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
            let (tx, rx) = unbounded();
            let file_path = Path::new(&self.output_directory)
                .join(format!("{}.fq.gz", output_filename));
            let file_directory = file_path.parent().unwrap();
            
            create_dir_all(&file_directory)
                .expect("Failed to create output directory");
            
            let file = File::create(&file_path)
                .expect("Failed to create output file");
            
            let encoder = GzEncoder::new(file, Compression::default());
            let writer = BufWriter::with_capacity(1_000_000, encoder);
            
            self.start_writing_thread(writer, rx);
            self.writers.insert(output_filename.clone(), tx);
        }
        
        self.writers.get(&output_filename).unwrap()
            .send(read_info)
            .expect("Failed to send read info to writer");
        
        Ok(())
    }

    fn start_writing_thread(&mut self, mut writer: BufWriter<GzEncoder<File>>, rx: Receiver<ReadInfo>) {
        let handle = thread::spawn(move || {
            for read_info in rx.iter() {
                if let Some(output_record) = read_info.get_output_record() {
                    let id = output_record.id();
                    let seq = std::str::from_utf8(output_record.seq())
                        .expect("Not a valid UTF-8 sequence");
                    let qual = std::str::from_utf8(output_record.qual())
                        .expect("Not a valid UTF-8 sequence");
                    let record_str = format!("@{}\n{}\n+\n{}\n", id, seq, qual);
                    write!(writer, "{}", record_str).unwrap();
                }
            }
            
            // Flush buffer
            writer.flush().expect("Failed to flush writer buffer");
            
            // Finish gzip encoder
            let encoder = writer.into_inner().expect("Failed to get inner encoder");
            encoder.finish().expect("Failed to finish gzip encoding");
        });
        
        self.thread_handles.push(handle);
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
    pub fn drop(&mut self) {
        info!("Writing FASTQ files, this may take some time...");
        
        // Drop all senders to close channels
        self.writers.clear();
        
        // Wait for all writing threads to complete
        for handle in self.thread_handles.drain(..) {
            handle.join().expect("Writing thread panicked");
        }
        
        info!("All writing threads completed successfully");
    }
    
}
