mod args;
mod pattern;
mod utils;
mod counter;
mod fastq;
mod myers;
mod splitter;
mod writer;
mod view;
mod thread_pool;

use clap::Parser;
use log::info;
use utils::ProcessInfo;
use thread_pool::{ThreadMonitor, ThreadAllocationStrategy};

#[tokio::main]
async fn main() {
    // Initialize logging system
    initialize_logging();
    
    // Parse command line arguments
    let args = args::Args::parse();
    info!("Starting ReadChop with command line arguments: {:?}", std::env::args().collect::<Vec<String>>());
    
    // Handle subcommands
    if let Some(command) = args.command {
        handle_subcommand(&command);
        return;
    }
    
    // Execute main sequence processing workflow
    execute_main_processing(&args).await;
}

/// Initialize logging system
fn initialize_logging() {
    unsafe {
    // Check if RUST_LOG is already set in environment
    if std::env::var("RUST_LOG").is_err() {
        // Only set to "info" if RUST_LOG is not already set
        std::env::set_var("RUST_LOG", "info");
    }
}
    pretty_env_logger::init();
}

/// Handle subcommands
fn handle_subcommand(command: &args::Commands) {
    match command {
        args::Commands::Encrypt { file } => {
            pattern::encrypt_pattern_database(&file, "666666");
        }
        args::Commands::View { .. } => {
            view::handle_view_command(command);
        }
    }
}

/// Execute main sequence processing workflow - memory optimized
async fn execute_main_processing(args: &args::Args) {
    let start_time = std::time::Instant::now();
    
    // Load pattern database
    let search_patterns = pattern::load_patterns(args);
    info!("Pattern database loaded successfully");
    
    // Create thread monitor with balanced allocation strategy
    let thread_strategy = ThreadAllocationStrategy::Balanced { 
        processing_ratio: 0.8  // 80% for processing, 20% for writing
    };
    let mut thread_monitor = ThreadMonitor::new(args.threads, thread_strategy);
    
    // Print thread allocation information
    thread_monitor.print_thread_stats();
    
    // Create FASTQ reader
    let read_receiver = fastq::create_reader(args.inputs.clone());
    
    // Create sequence splitter with controlled thread count
    let split_receiver = splitter::create_splitter_receiver_controlled(
        read_receiver, 
        &search_patterns, 
        thread_monitor.get_processing_threads(),
        &mut thread_monitor.thread_pool
    );
    
    // Initialize statistics and write manager
    let mut statistics_manager = counter::StatisticsManager::new(args.outdir.clone());
    let mut file_writer_manager = writer::FileWriterManager::new(
        args.outdir.clone(), 
        thread_monitor.get_writing_threads(),
        args.enable_logger
    );
    let mut progress_tracker = ProcessInfo::new(args.log_interval);
    
    // Process each sequence
    for read_info in split_receiver {
        // Create lightweight stats copy for statistics
        let read_stats = read_info.create_stats_copy();
        
        // Log record (only if logger is enabled)
        file_writer_manager.log(read_info.to_tsv());
        
        // Update statistics using lightweight structure
        statistics_manager.process_read_stats(&read_stats);
        
        // Write file (blocking if channel is full to control memory usage)
        file_writer_manager.write(read_info).await
            .expect("Failed to write sequence information");
        
        // Update progress
        progress_tracker.info();
    }
    
    // Splitter completed, release processing threads to writer
    info!("Splitter processing completed, releasing {} threads to writer", 
          thread_monitor.get_processing_threads());
    file_writer_manager.expand_concurrency(thread_monitor.get_processing_threads());
    
    // Complete processing
    finalize_processing(
        &mut file_writer_manager,
        &statistics_manager,
        start_time,
        &args.outdir
    ).await;
}

/// Complete processing and output results
async fn finalize_processing(
    file_writer_manager: &mut writer::FileWriterManager,
    statistics_manager: &counter::StatisticsManager,
    start_time: std::time::Instant,
    output_dir: &str,
) {
    // Write log file
    file_writer_manager.write_log_file(output_dir).await
        .expect("Failed to write log file");
    
    // Write statistics
    statistics_manager.write_total_statistics();
    statistics_manager.write_valid_statistics();
    
    // Output statistics
    statistics_manager.print_statistics();
    
    let processing_time = start_time.elapsed();
    info!("Sequence splitting completed! Processing time: {:.4?}", processing_time);
    
    // Wait for all write tasks to complete
    file_writer_manager.finish().await;
    
    let total_time = start_time.elapsed();
    info!("All processing completed! Total time: {:.4?}", total_time);
}
