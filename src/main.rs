mod args;
mod pattern;
mod utils;
mod counter;
mod fastq;
mod myers;
mod splitter;
mod writer;
mod view;

use clap::Parser;
use log::info;
use utils::ProcessInfo;

fn main() {
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
    execute_main_processing(&args);
}

/// Initialize logging system
fn initialize_logging() {
    unsafe {
        std::env::set_var("RUST_LOG", "info");
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

/// Execute main sequence processing workflow
fn execute_main_processing(args: &args::Args) {
    let start_time = std::time::Instant::now();
    
    // Load pattern database
    let search_patterns = pattern::load_patterns(args);
    info!("Pattern database loaded successfully");
    
    // Create FASTQ reader
    let read_receiver = fastq::create_reader(args.inputs.clone());
    
    // Create sequence splitter
    let split_receiver = splitter::create_splitter_receiver(
        read_receiver, 
        &search_patterns, 
        args.threads
    );
    
    // Initialize statistics and write manager
    let mut statistics_manager = counter::StatisticsManager::new(args.outdir.clone());
    let mut file_writer_manager = writer::FileWriterManager::new(args.outdir.clone());
    let mut progress_tracker = ProcessInfo::new(args.log_interval);
    
    // Process each sequence
    for read_info in split_receiver {
        // Log record
        file_writer_manager.logger.push(read_info.to_tsv());
        
        // Update statistics
        statistics_manager.process_read(&read_info);
        
        // Write file
        file_writer_manager.write(read_info)
            .expect("Failed to write sequence information");
        
        // Update progress
        progress_tracker.info();
    }
    
    // Complete processing
    finalize_processing(
        &mut file_writer_manager,
        &statistics_manager,
        start_time,
        &args.outdir
    );
}

/// Complete processing and output results
fn finalize_processing(
    file_writer_manager: &mut writer::FileWriterManager,
    statistics_manager: &counter::StatisticsManager,
    start_time: std::time::Instant,
    output_dir: &str,
) {
    // Write log file
    file_writer_manager.write_log_file(output_dir)
        .expect("Failed to write log file");
    
    // Write statistics
    statistics_manager.write_total_statistics();
    statistics_manager.write_valid_statistics();
    
    // Output statistics
    statistics_manager.print_statistics();
    
    let processing_time = start_time.elapsed();
    info!("Sequence splitting completed! Processing time: {:.4?}", processing_time);
    
    // Wait for all write threads to complete
    file_writer_manager.finalize();
    
    let total_time = start_time.elapsed();
    info!("All processing completed! Total time: {:.4?}", total_time);
}
