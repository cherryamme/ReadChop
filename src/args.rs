use clap::builder::styling::{AnsiColor, Effects, Styles};
use clap::{Parser, Subcommand};

/// Create CLI style configuration
fn create_cli_styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Yellow.on_default() | Effects::BOLD)
        .usage(AnsiColor::Yellow.on_default() | Effects::BOLD)
        .literal(AnsiColor::Blue.on_default() | Effects::BOLD)
        .placeholder(AnsiColor::Green.on_default())
}

/// Main command line arguments structure
#[derive(Parser, Debug, Clone)]
#[command(
    help_template = "{usage-heading} {usage} \nVersion: {version} {about-section}Author:{author} Email: cherryamme@qq.com\n {all-args} {tab}"
)]
#[command(
    version, 
    author, 
    about, 
    long_about = None, 
    styles = create_cli_styles(), 
    subcommand_negates_reqs = true, 
    args_conflicts_with_subcommands = true
)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Input file paths
    #[arg(short, long, num_args = 1.., value_delimiter = ' ')]
    pub inputs: Vec<String>,
    
    /// Output directory name
    #[arg(short, long, default_value = "outdir")]
    pub outdir: String,
    
    /// Number of threads
    #[arg(short, long, default_value = "20")]
    pub threads: usize,
    
    /// Minimum sequence length filter threshold
    #[arg(short, long, default_value = "100")]
    pub min_length: usize,
    
    /// Pattern file list
    #[arg(short, long, required = true, num_args = 1.., value_delimiter = ' ')]
    pub pattern_files: Option<Vec<String>>,
    
    /// Pattern database file
    #[arg(short = 'd', long = "db", required = true)]
    pub pattern_db_file: Option<String>,
    
    /// Fusion detection file
    #[arg(short = 'f', long = "fusion", default_value = "")]
    pub fusion_file: String,
    
    /// Fusion detection error rate
    #[arg(long = "fe", default_value = "0.2")]
    pub fusion_error_rate: f32,
    
    /// Log recording interval
    #[arg(short = 'n', long = "num", default_value = "500000")]
    pub log_interval: u32,
    
    /// Search window size <left window, right window>
    #[arg(short, long, value_delimiter = ',', default_value = "400,400")]
    pub window_size: Vec<usize>,
    
    /// Pattern matching error rate <left error rate, right error rate>, range 0-0.5
    #[arg(short = 'e', long, num_args = 1.., value_delimiter = ' ', default_value = "0.2,0.2", value_parser = validate_error_rate)]
    pub pattern_error_rate: Vec<(f32, f32)>,
    
    /// Sequence trimming mode: 0=trim all, 1=keep one pattern, 2=keep two patterns...
    #[arg(long, default_value = "0")]
    pub trim_mode: usize,
    
    /// Write type: names=use names, type=use types
    #[arg(long, default_value = "type", value_parser = ["names", "type"])]
    pub write_type: String,
    
    /// Pattern matching type: single=single pattern, dual=dual pattern
    #[arg(long = "match", num_args = 1.., value_delimiter = ' ', default_value = "single", value_parser = ["single", "dual"])]
    pub pattern_match_type: Vec<String>,
    
    /// Whether to use position information for more precise detection
    #[arg(long = "pos")]
    pub use_position_info: bool,
    
    /// Position offset for multi-pattern splitting
    #[arg(long = "shift", num_args = 1.., value_delimiter = ' ', default_value = "3")]
    pub position_shift: Vec<usize>,
    
    /// Maximum distance threshold
    #[arg(long = "maxdist", num_args = 1.., value_delimiter = ',', default_value = "4")]
    pub max_distance: Vec<usize>,
    
    /// Record ID separator
    #[arg(long = "id_sep", default_value = "%")]
    pub id_separator: String,
}

/// Subcommand enumeration
#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Encrypt database file
    Encrypt {
        /// Database file to encrypt
        file: String,
    },
    /// Preview barcode detection results (with color highlighting)
    View {
        /// Input file paths
        #[arg(short, long, num_args = 1.., value_delimiter = ' ')]
        inputs: Vec<String>,
        /// Pattern file list
        #[arg(short, long, required = true, num_args = 1.., value_delimiter = ' ')]
        pattern_files: Vec<String>,
        /// Pattern database file
        #[arg(short = 'd', long = "db", required = true)]
        pattern_db_file: String,
        /// Number of threads
        #[arg(short, long, default_value = "20")]
        threads: usize,
        /// Minimum sequence length filter threshold
        #[arg(short, long, default_value = "100")]
        min_length: usize,
        /// Search window size <left window, right window>
        #[arg(short, long, value_delimiter = ',', default_value = "400,400")]
        window_size: Vec<usize>,
        /// Pattern matching error rate <left error rate, right error rate>, range 0-0.5
        #[arg(short = 'e', long, num_args = 1.., value_delimiter = ' ', default_value = "0.2,0.2", value_parser = validate_error_rate)]
        pattern_error_rate: Vec<(f32, f32)>,
        /// Sequence trimming mode: 0=trim all, 1=keep one pattern, 2=keep two patterns...
        #[arg(long, default_value = "0")]
        trim_mode: usize,
        /// Pattern matching type: single=single pattern, dual=dual pattern
        #[arg(long = "match", num_args = 1.., value_delimiter = ' ', default_value = "single", value_parser = ["single", "dual"])]
        pattern_match_type: Vec<String>,
        /// Whether to use position information for more precise detection
        #[arg(long = "pos")]
        use_position_info: bool,
        /// Position offset for multi-pattern splitting
        #[arg(long = "shift", num_args = 1.., value_delimiter = ' ', default_value = "3")]
        position_shift: Vec<usize>,
        /// Maximum distance threshold
        #[arg(long = "maxdist", num_args = 1.., value_delimiter = ',', default_value = "4")]
        max_distance: Vec<usize>,
        /// Record ID separator
        #[arg(long = "id_sep", default_value = "%")]
        id_separator: String,
    },
}

/// Validate error rate parameters
fn validate_error_rate(input: &str) -> Result<(f32, f32), String> {
    let error_rates: Vec<&str> = input.split(',').collect();
    
    if error_rates.len() != 2 {
        return Err("Error rate parameter should contain two comma-separated values".to_string());
    }
    
    let left_rate = error_rates[0].parse::<f32>();
    let right_rate = error_rates[1].parse::<f32>();
    
    match (left_rate, right_rate) {
        (Ok(left), Ok(right)) if left >= 0.0 && left <= 0.5 && right >= 0.0 && right <= 0.5 => {
            Ok((left, right))
        },
        _ => Err("Error rate parameter error. Should be a floating point number between 0 and 0.5.".to_string()),
    }
}

impl Args {
    /// Get pattern file list, return empty vector if None
    pub fn get_pattern_files(&self) -> Vec<String> {
        self.pattern_files.clone().unwrap_or_default()
    }
    
    /// Get pattern database file path, return empty string if None
    pub fn get_pattern_db_file(&self) -> String {
        self.pattern_db_file.clone().unwrap_or_default()
    }
    
    /// Check if fusion detection is enabled
    pub fn is_fusion_detection_enabled(&self) -> bool {
        !self.fusion_file.is_empty()
    }
    
    /// Get minimum sequence length, ensure at least 1
    pub fn get_min_length(&self) -> usize {
        self.min_length.max(1)
    }
}