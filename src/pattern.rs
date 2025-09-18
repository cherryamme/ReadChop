use csv;
use log::info;
use std::collections::HashMap;
use crate::args::Args;
use crate::utils::reverse_complement;
use age::secrecy::SecretString;
use std::fs::File;
use std::io::{Read, Write};

/// Pattern parameter configuration structure
#[derive(Debug, Clone)]
pub struct PatternConfiguration {
    pub window_size: Vec<usize>,
    pub pattern_match_types: Vec<String>,
    pub pattern_arguments: Vec<PatternArgument>,
    pub trim_mode: usize,
    pub write_type: String,
    pub pattern_error_rates: Vec<(f32, f32)>,
    pub max_distances: Vec<usize>,
    pub position_shifts: Vec<usize>,
    pub min_length: usize,
    pub id_separator: String,
    pub fusion_database: FusionDatabase,
    pub fusion_error_rate: f32,
}

impl PatternConfiguration {
    /// Create pattern configuration from command line arguments
    pub fn new(args: &Args) -> Self {
        let mut config = Self {
            window_size: args.window_size.clone(),
            pattern_match_types: args.pattern_match_type.clone(),
            pattern_arguments: vec![],
            trim_mode: args.trim_mode,
            write_type: args.write_type.clone(),
            pattern_error_rates: args.pattern_error_rate.clone(),
            max_distances: args.max_distance.clone(),
            position_shifts: args.position_shift.clone(),
            min_length: args.get_min_length(),
            id_separator: args.id_separator.clone(),
            fusion_database: FusionDatabase::new(),
            fusion_error_rate: args.fusion_error_rate,
        };
        config.normalize_vectors();
        config
    }
    
    /// Normalize vector length
    pub fn normalize_vectors(&mut self) {
        const MIN_VECTOR_LENGTH: usize = 5;
        
        Self::resize_vector(&mut self.pattern_match_types, MIN_VECTOR_LENGTH);
        Self::resize_vector(&mut self.pattern_error_rates, MIN_VECTOR_LENGTH);
        Self::resize_vector(&mut self.max_distances, MIN_VECTOR_LENGTH);
        Self::resize_vector(&mut self.position_shifts, MIN_VECTOR_LENGTH);
    }
    
    /// Adjust vector to minimum length
    fn resize_vector<T: Clone + Default>(vector: &mut Vec<T>, min_length: usize) {
        if vector.len() < min_length {
            let last_element = vector.last().cloned().unwrap_or_default();
            vector.resize(min_length, last_element);
        }
    }
}

/// Single pattern parameter
#[derive(Debug, Clone)]
pub struct PatternArgument {
    pub pattern_database: PatternDatabase,
    pub use_position_info: bool,
    pub pattern_error_rate: (f32, f32),
    pub max_distance: usize,
    pub position_shift: usize,
}

/// Encrypt pattern database file
pub fn encrypt_pattern_database(file_path: &str, passphrase: &str) {
    let mut file = File::open(file_path)
        .expect(&format!("Unable to find file: {}", file_path));
    
    let mut content = Vec::new();
    file.read_to_end(&mut content)
        .expect("Failed to read file content");

    // Encrypt content
    let secret_passphrase = SecretString::from(passphrase.to_owned());
    let recipient = age::scrypt::Recipient::new(secret_passphrase);
    let encrypted_data = age::encrypt(&recipient, &content)
        .expect("Failed to encrypt data");

    // Write encrypted file
    let output_file = format!("{}.safe", file_path);
    let mut output_file_handle = File::create(&output_file)
        .expect("Failed to create encrypted file");
    output_file_handle.write_all(&encrypted_data)
        .expect("Failed to write encrypted data");
    
    info!("Pattern database file encrypted and saved to: {}", output_file);
}

/// Pattern database structure
#[derive(Debug, Clone)]
pub struct PatternDatabase {
    /// Forward patterns
    pub forward_patterns: HashMap<String, String>,
    /// Reverse patterns
    pub reverse_patterns: HashMap<String, String>,
    /// Pattern type mapping
    pub pattern_types: HashMap<String, (String, String, String)>,
}

impl PatternDatabase {
    /// Create new pattern database
    pub fn new() -> Self {
        Self {
            forward_patterns: HashMap::new(),
            reverse_patterns: HashMap::new(),
            pattern_types: HashMap::new(),
        }
    }
    
    /// Load pattern data
    pub fn load_patterns(&mut self, database_file: &str, pattern_file: &str) {
        let pattern_database = self.load_database(database_file, "666666");
        self.load_pattern_file(pattern_file, pattern_database);
    }
    
    /// Load database file
    fn load_database(&self, file_path: &str, passphrase: &str) -> HashMap<String, String> {
        let mut pattern_database = HashMap::new();
        let mut content = Vec::new();

        if file_path.ends_with(".safe") {
            // Decrypt file
            let secret_passphrase = SecretString::from(passphrase.to_owned());
            let identity = age::scrypt::Identity::new(secret_passphrase);
            let mut encrypted_file = File::open(file_path)
                .expect(&format!("Unable to find encrypted file: {}", file_path));
            encrypted_file.read_to_end(&mut content)
                .expect("Failed to read encrypted file");
            let decrypted_data = age::decrypt(&identity, &content[..])
                .expect("Failed to decrypt file");
            content = decrypted_data;
        } else {
            // Read file directly
            let mut file = File::open(file_path)
                .expect(&format!("Unable to find file: {}", file_path));
            file.read_to_end(&mut content)
                .expect("Failed to read file");
        }

        let cursor = std::io::Cursor::new(content);
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .delimiter(b'\t')
            .from_reader(cursor);

        for result in reader.records() {
            let record = result.expect("Failed to parse CSV record");
            let name = &record[0];
            let sequence = &record[1];
            pattern_database.insert(name.to_string(), sequence.to_string());
        }
        
        pattern_database
    }
    
    /// Load pattern files
    fn load_pattern_file(&mut self, file_path: &str, pattern_database: HashMap<String, String>) {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .delimiter(b'\t')
            .from_path(file_path)
            .expect(&format!("Unable to find pattern file: {}", file_path));
            
        for result in reader.records() {
            let record = result.expect("Failed to parse pattern file record");
            let (forward_key, reverse_key, name) = (
                record[0].to_string(), 
                record[1].to_string(), 
                record[2].to_string()
            );
            
            let forward_reverse_key = format!("{}_{}", forward_key, reverse_key);
            let reverse_forward_key = format!("{}_{}", reverse_key, forward_key);
            
            let forward_sequence = pattern_database
                .get(&forward_key)
                .expect(&format!("Pattern not found in database: {}", forward_key))
                .to_string();
            let reverse_sequence = pattern_database
                .get(&reverse_key)
                .expect(&format!("Pattern not found in database: {}", reverse_key))
                .to_string();
            
            // Store forward and reverse patterns
            self.forward_patterns.insert(forward_key.clone(), forward_sequence.clone());
            self.forward_patterns.insert(reverse_key.clone(), reverse_sequence.clone());
            self.reverse_patterns.insert(forward_key.clone(), reverse_complement(&forward_sequence));
            self.reverse_patterns.insert(reverse_key.clone(), reverse_complement(&reverse_sequence));
            
            // Store pattern type information
            if forward_reverse_key != reverse_forward_key {
                self.pattern_types.insert(
                    forward_reverse_key.clone(), 
                    (forward_reverse_key.clone(), name.clone(), "fs".to_string())
                );
                self.pattern_types.insert(
                    reverse_forward_key.clone(), 
                    (forward_reverse_key, name, "rs".to_string())
                );
            } else {
                self.pattern_types.insert(
                    forward_reverse_key.clone(), 
                    (forward_reverse_key, name, "unknown".to_string())
                );
            }
        }
        
        info!("Pattern file loaded successfully: {}", file_path);
    }
}

/// Fusion database structure
#[derive(Debug, Clone)]
pub struct FusionDatabase {
    pub fusion_patterns: HashMap<String, String>,
}

impl FusionDatabase {
    /// 创建新的融合数据库
    pub fn new() -> Self {
        Self {
            fusion_patterns: HashMap::new(),
        }
    }
    
    /// 检查数据库是否为空
    pub fn is_empty(&self) -> bool {
        self.fusion_patterns.is_empty()
    }
    
    /// 加载融合模式数据
    pub fn load_fusion_patterns(&mut self, database_file: &str, fusion_file: &str) {
        let pattern_database = self.load_database(database_file);
        self.load_fusion_file(fusion_file, pattern_database);
    }
    
    /// Load database file
    fn load_database(&self, file_path: &str) -> HashMap<String, String> {
        let mut pattern_database = HashMap::new();
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .delimiter(b'\t')
            .from_path(file_path)
            .expect(&format!("Unable to find database file: {}", file_path));
            
        for result in reader.records() {
            let record = result.expect("Failed to parse database record");
            let name = &record[0];
            let sequence = &record[1];
            pattern_database.insert(name.to_string(), sequence.to_string());
        }
        
        pattern_database
    }
    
    /// 加载融合文件
    fn load_fusion_file(&mut self, file_path: &str, pattern_database: HashMap<String, String>) {
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .delimiter(b'\t')
            .from_path(file_path)
            .expect(&format!("Unable to find fusion file: {}", file_path));
            
        for result in reader.records() {
            let record = result.expect("Failed to parse fusion file record");
            let fusion_pattern = record[0].to_string();
            let fusion_sequence = pattern_database
                .get(&fusion_pattern)
                .expect(&format!("Fusion pattern not found in database: {}", fusion_pattern))
                .to_string();
            self.fusion_patterns.insert(fusion_pattern, fusion_sequence);
        }
    }
}

/// 加载模式配置
pub fn load_patterns(args: &Args) -> PatternConfiguration {
    info!("Loading pattern database file: {}", args.get_pattern_db_file());
    
    let mut pattern_config = PatternConfiguration::new(args);
    
    // 加载融合数据库
    if args.is_fusion_detection_enabled() {
        pattern_config.fusion_database.load_fusion_patterns(
            &args.get_pattern_db_file(), 
            &args.fusion_file
        );
    }
    
    // 加载模式文件
    for pattern_file in args.get_pattern_files() {
        let mut pattern_database = PatternDatabase::new();
        pattern_database.load_patterns(&args.get_pattern_db_file(), &pattern_file);
        
        let pattern_argument = PatternArgument {
            pattern_database,
            use_position_info: args.use_position_info,
            pattern_error_rate: pattern_config.pattern_error_rates[0],
            max_distance: pattern_config.max_distances[0],
            position_shift: pattern_config.position_shifts[0],
        };
        pattern_config.pattern_arguments.push(pattern_argument);
    }
    
    pattern_config
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pattern_configuration_creation() {
        // 这里可以添加测试代码
    }
    
    #[test]
    fn test_pattern_database_loading() {
        // 这里可以添加测试代码
    }
}