use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use log::info;
use crate::fastq::ReadInfo;
use std::io::Write;

/// Statistics manager structure
pub struct StatisticsManager {
    /// Basic counter
    pub counters: HashMap<String, u32>,
    /// Valid name counter
    pub valid_name_counters: HashMap<String, HashMap<String, HashMap<String, u32>>>,
    /// Valid type counter
    pub valid_type_counters: HashMap<String, HashMap<String, HashMap<String, u32>>>,
    /// Output directory
    output_directory: String,
    /// Total reads
    total_reads: u32,
    /// Total bases
    total_bases: u32,
    /// Pre-processing GC content
    before_gc_content: f64,
    /// Valid reads
    valid_reads: u32,
    /// Valid bases
    valid_bases: u32,
    /// Post-processing GC content
    after_gc_content: f64,
}

impl StatisticsManager {
    /// Create new statistics manager
    pub fn new(output_directory: String) -> Self {
        info!("Creating statistics manager, starting counting...");
        
        let mut counters = HashMap::new();
        counters.insert("filtered".to_string(), 0);
        counters.insert("unknown".to_string(), 0);
        counters.insert("fusion".to_string(), 0);
        
        Self {
            counters,
            valid_name_counters: HashMap::new(),
            valid_type_counters: HashMap::new(),
            output_directory,
            total_reads: 0,
            total_bases: 0,
            before_gc_content: 0.5,
            valid_reads: 0,
            valid_bases: 0,
            after_gc_content: 0.5,
        }
    }
    
    /// Process single read
    pub fn process_read(&mut self, read_info: &ReadInfo) {
        self.total_reads += 1;
        self.total_bases += read_info.sequence_length as u32;
        
        // Update basic counter
        *self.counters.entry(read_info.sequence_type.clone()).or_insert(0) += 1;
        
        // If valid sequence, perform detailed statistics
        if read_info.sequence_type == "valid" {
            self.valid_reads += 1;
            self.valid_bases += read_info.sequence_length as u32;
            self.update_detailed_statistics(read_info);
        }
    }
    
    /// Update detailed statistics
    fn update_detailed_statistics(&mut self, read_info: &ReadInfo) {
        let primer = read_info.match_names[0].clone();
        let index = read_info.match_names[1].clone();
        let barcode = read_info.match_names[2].clone();
        let primer_type = read_info.match_types[0].clone();
        let index_type = read_info.match_types[1].clone();
        let barcode_type = read_info.match_types[2].clone();
        
        // Update name counter
        let barcode_map = self.valid_name_counters
            .entry(barcode.clone())
            .or_insert_with(HashMap::new);
        let index_map = barcode_map.entry(index.clone()).or_insert_with(HashMap::new);
        *index_map.entry(primer).or_insert(0) += 1;
        
        // Update type counter
        let barcode_type_map = self.valid_type_counters
            .entry(barcode_type)
            .or_insert_with(HashMap::new);
        let index_type_map = barcode_type_map.entry(index_type).or_insert_with(HashMap::new);
        *index_type_map.entry(primer_type).or_insert(0) += 1;
    }
    
    /// Write valid statistics
    pub fn write_valid_statistics(&self) {
        self.write_name_statistics();
        self.write_type_statistics();
    }
    
    /// Write name statistics
    fn write_name_statistics(&self) {
        for (barcode, index_map) in &self.valid_name_counters {
            let file_path = Path::new(&self.output_directory)
                .join(format!("{}_validname.tsv", barcode));
            let mut file = File::create(&file_path)
                .expect("Failed to create valid name statistics file");
            
            writeln!(file, "barcode\tindex\tprimer\tcount")
                .expect("Failed to write table header");
            
            for (index, primer_map) in index_map {
                for (primer, count) in primer_map {
                    writeln!(file, "{}\t{}\t{}\t{}", barcode, index, primer, count)
                        .expect("Failed to write valid name statistics");
                }
            }
        }
    }
    
    /// Write type statistics
    fn write_type_statistics(&self) {
        for (barcode, index_map) in &self.valid_type_counters {
            let file_path = Path::new(&self.output_directory)
                .join(format!("{}_validtype.tsv", barcode));
            let mut file = File::create(&file_path)
                .expect("Failed to create valid type statistics file");
            
            writeln!(file, "barcode\tindex\tprimer\tcount")
                .expect("Failed to write table header");
            
            for (index, primer_map) in index_map {
                for (primer, count) in primer_map {
                    writeln!(file, "{}\t{}\t{}\t{}", barcode, index, primer, count)
                        .expect("Failed to write valid type statistics");
                }
            }
        }
    }
    
    /// Print statistics
    pub fn print_statistics(&self) {
        let valid_reads = self.valid_reads as f64;
        let total_reads = self.total_reads as f64;
        let fusion_count = self.counters.get("fusion").unwrap_or(&0);
        let filtered_count = self.counters.get("filtered").unwrap_or(&0);
        
        let valid_rate = if total_reads > 0.0 {
            100.0 * valid_reads / total_reads
        } else {
            0.0
        };
        
        let filtered_rate = if total_reads > 0.0 {
            100.0 * *filtered_count as f64 / total_reads
        } else {
            0.0
        };
        
        let fusion_rate = if total_reads > 0.0 {
            100.0 * *fusion_count as f64 / total_reads
        } else {
            0.0
        };
        
        info!(
            "Processed {}/{} reads (filtered/total), filter rate: {:.2}%", 
            filtered_count, total_reads, filtered_rate
        );
        info!(
            "Processed {}/{} reads (fusion/total), fusion rate: {:.2}%", 
            fusion_count, total_reads, fusion_rate
        );
        info!(
            "Processed {}/{} reads (valid/total), valid rate: {:.2}%", 
            valid_reads, total_reads, valid_rate
        );
    }
    
    /// Write total statistics
    pub fn write_total_statistics(&self) {
        let total_reads = self.total_reads as f64;
        let valid_reads = self.valid_reads as f64;
        let total_bases = self.total_bases as f64;
        let valid_bases = self.valid_bases as f64;

        let before_mean_length = if total_reads > 0.0 {
            total_bases / total_reads
        } else {
            0.0
        };
        
        let after_mean_length = if valid_reads > 0.0 {
            valid_bases / valid_reads
        } else {
            0.0
        };
        
        let valid_count = *self.counters.get("valid").unwrap_or(&0) as f64;
        let unknown_count = *self.counters.get("unknown").unwrap_or(&0) as f64;
        let filtered_count = *self.counters.get("filtered").unwrap_or(&0) as f64;
        let fusion_count = *self.counters.get("fusion").unwrap_or(&0) as f64;

        let valid_rate = if total_reads > 0.0 {
            valid_count / total_reads * 100.0
        } else {
            0.0
        };
        
        let unknown_rate = if total_reads > 0.0 {
            unknown_count / total_reads * 100.0
        } else {
            0.0
        };
        
        let filtered_rate = if total_reads > 0.0 {
            filtered_count / total_reads * 100.0
        } else {
            0.0
        };
        
        let fusion_rate = if total_reads > 0.0 {
            fusion_count / total_reads * 100.0
        } else {
            0.0
        };

        let file_path = Path::new(&self.output_directory).join("total_info.tsv");
        let mut file = File::create(&file_path)
            .expect("Failed to create total statistics file");
        
        writeln!(
            file, 
            "total\ttotal_bases\tbefore_read1_mean_length\tafter_read1_mean_length\tbefore_gc_content\tafter_gc_content\tfiltered\tfiltered_rate\tfusion\tfusion_rate\tunknown\tunknown_rate\tvalid_reads\tvalid_bases\tvalid_rate"
        ).expect("Failed to write header");
        
        writeln!(
            file,
            "{}\t{}\t{:.1}\t{:.1}\t{:.1}\t{:.1}\t{}\t{:.2}\t{}\t{:.2}\t{}\t{:.2}\t{}\t{}\t{:.2}",
            total_reads as u32,
            total_bases as u32,
            before_mean_length,
            after_mean_length,
            self.before_gc_content,
            self.after_gc_content,
            filtered_count as u32,
            filtered_rate,
            fusion_count as u32,
            fusion_rate,
            unknown_count as u32,
            unknown_rate,
            valid_count as u32,
            valid_bases as u32,
            valid_rate,
        ).expect("Failed to write total statistics");
    }
    
}