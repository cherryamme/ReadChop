use crate::args::Commands;
use crate::fastq::ReadInfo;
use crate::pattern::PatternConfiguration;
use crate::splitter::perform_sequence_splitting_vector;
use flume::Receiver;
use log::info;

/// 处理view子命令，实时预览条形码识别结果
pub fn handle_view_command(view_args: &Commands) {
    info!("Starting preview mode, displaying barcode recognition results in real-time");
    
    // 构建模式配置
    let pattern_config = PatternConfiguration::new_from_view_args(view_args);
    
    // 创建FASTQ读取器
    let inputs = match view_args {
        Commands::View { inputs, .. } => inputs.clone(),
        _ => return,
    };
    let read_receiver: Receiver<ReadInfo> = crate::fastq::create_reader(inputs);
    
    // 处理每个序列
    for read_info in read_receiver.iter() {
        // 执行条形码识别
        let split_types = perform_sequence_splitting_vector(&read_info, &pattern_config);
        
        // 输出结果
        print_sequence_result(&read_info, &split_types);
    }
}

/// 打印单个序列的识别结果，包含颜色高亮
fn print_sequence_result(read_info: &ReadInfo, split_types: &[crate::splitter::SplitType]) {
    // 输出序列ID和长度
    println!("Sequence ID: {} Length: {}", read_info.record.id(), read_info.record.seq().len());
    
    // 获取序列
    let sequence = read_info.record.seq();
    let mut barcode_positions = Vec::new();
    
    // 收集所有检测到的条形码位置
    for split_type in split_types {
        if split_type.left_matcher.status {
            barcode_positions.push((
                split_type.left_matcher.ystart,
                split_type.left_matcher.yend,
                &split_type.pattern_name,
                split_type.left_matcher.get_score(),
            ));
        }
        if split_type.right_matcher.status {
            barcode_positions.push((
                split_type.right_matcher.ystart,
                split_type.right_matcher.yend,
                &split_type.pattern_name,
                split_type.right_matcher.get_score(),
            ));
        }
    }
    
    // 按位置排序
    barcode_positions.sort_by_key(|x| x.0);
    
    // 构建高亮序列
    let red_start = "\x1b[31m";  // 红色开始
    let red_end = "\x1b[0m";     // 颜色结束
    let mut highlighted_sequence = String::new();
    let mut last_position = 0;
    
    for (start, end, _pattern_name, _errors) in &barcode_positions {
        // 添加条形码前的序列
        if *start > last_position {
            highlighted_sequence.push_str(&String::from_utf8_lossy(&sequence[last_position..*start]));
        }
        
        // 添加红色高亮的条形码
        let barcode_sequence = &sequence[*start..*end];
        highlighted_sequence.push_str(&format!(
            "{}{}{}",
            red_start,
            String::from_utf8_lossy(barcode_sequence),
            red_end
        ));
        
        last_position = *end;
    }
    
    // 添加剩余序列
    if last_position < sequence.len() {
        highlighted_sequence.push_str(&String::from_utf8_lossy(&sequence[last_position..]));
    }
    
    // 智能截断：保持ANSI转义序列的完整性
    if highlighted_sequence.len() > 200 {
        let truncated = smart_truncate_preserve_ansi(&highlighted_sequence, 200);
        println!("Sequence: {}", truncated);
    } else {
        println!("Sequence: {}", highlighted_sequence);
    }
    
    // 输出检测到的模式信息
    print!("Detected patterns: ");
    for (i, split_type) in split_types.iter().enumerate() {
        if i > 0 {
            print!(" ");
        }
        
        if split_type.left_matcher.status {
            print!("({},{},{},{})", 
                split_type.pattern_name,
                split_type.left_matcher.get_score(),
                split_type.left_matcher.ystart,
                split_type.left_matcher.yend
            );
        }
        
        if split_type.right_matcher.status {
            if split_type.left_matcher.status {
                print!(" ");
            }
            print!("({},{},{},{})", 
                split_type.pattern_name,
                split_type.right_matcher.get_score(),
                split_type.right_matcher.ystart,
                split_type.right_matcher.yend
            );
        }
    }
    println!();
    println!(); // 空行分隔
}

/// 智能截断字符串，保持ANSI转义序列的完整性
fn smart_truncate_preserve_ansi(text: &str, max_length: usize) -> String {
    if text.len() <= max_length {
        return text.to_string();
    }
    
    // 简单截断：取前100个字符 + "..." + 后100个字符
    let front_length = 100;
    let back_length = 100;
    
    if text.len() <= front_length + back_length + 3 {
        return text.to_string();
    }
    
    let front = &text[..front_length];
    let back = &text[text.len()-back_length..];
    
    format!("{}...{}", front, back)
}

impl PatternConfiguration {
    /// 从View命令参数创建模式配置
    pub fn new_from_view_args(view_args: &Commands) -> PatternConfiguration {
        let (window_size, pattern_match_types, trim_mode, pattern_error_rates, 
             max_distances, position_shifts, min_length, id_separator, 
             pattern_db_file, pattern_files, use_position_info) = match view_args {
            Commands::View { 
                window_size, 
                pattern_match_type, 
                trim_mode, 
                pattern_error_rate, 
                max_distance, 
                position_shift, 
                min_length, 
                id_separator, 
                pattern_db_file, 
                pattern_files, 
                use_position_info, 
                .. 
            } => (
                window_size.clone(), 
                pattern_match_type.clone(), 
                *trim_mode, 
                pattern_error_rate.clone(), 
                max_distance.clone(), 
                position_shift.clone(), 
                *min_length, 
                id_separator.clone(), 
                pattern_db_file.clone(), 
                pattern_files.clone(), 
                *use_position_info
            ),
            _ => return PatternConfiguration {
                window_size: vec![400, 400],
                pattern_match_types: vec!["single".to_string()],
                pattern_arguments: vec![],
                trim_mode: 0,
                write_type: "names".to_string(),
                pattern_error_rates: vec![(0.2, 0.2)],
                max_distances: vec![4],
                position_shifts: vec![3],
                min_length: 100,
                id_separator: "%".to_string(),
                fusion_database: crate::pattern::FusionDatabase::new(),
                fusion_error_rate: 0.2,
            },
        };
        
        let mut pattern_config = PatternConfiguration {
            window_size,
            pattern_match_types,
            pattern_arguments: vec![],
            trim_mode,
            write_type: "names".to_string(), // view模式不需要写入文件
            pattern_error_rates,
            max_distances,
            position_shifts,
            min_length,
            id_separator,
            fusion_database: crate::pattern::FusionDatabase::new(),
            fusion_error_rate: 0.2,
        };
        
        pattern_config.normalize_vectors();
        
        // 加载模式数据库
        info!("Loading pattern database file: {}", pattern_db_file);
        for pattern_file in &pattern_files {
            let mut pattern_database = crate::pattern::PatternDatabase::new();
            pattern_database.load_patterns(&pattern_db_file, pattern_file);
            
            let pattern_argument = crate::pattern::PatternArgument {
                pattern_database,
                use_position_info,
                pattern_error_rate: pattern_config.pattern_error_rates[0],
                max_distance: pattern_config.max_distances[0],
                position_shift: pattern_config.position_shifts[0],
            };
            pattern_config.pattern_arguments.push(pattern_argument);
        }
        
        pattern_config
    }
}