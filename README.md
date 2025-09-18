# ReadChop

ReadChop is a command-line tool designed for third-generation sequencing data, used to split long-read FASTQ/GZ files based on specified patterns.

## Features

- **Efficient Demultiplexing**: Supports sequence splitting based on barcode patterns
- **Multi-threading**: Supports multi-threaded parallel processing for improved speed
- **Flexible Matching**: Supports both single and dual pattern matching
- **Error Tolerance**: Configurable matching error rates
- **Preview Function**: Supports preview of splitting results with color highlighting
- **Database Encryption**: Supports pattern database file encryption

## Installation

### Build from Source

```bash
# Clone repository
git clone <repository-url>
cd ReadChop

# Build release version
cargo build --release

# Executable file is located at target/release/readchop
```

### Requirements

- Rust 1.70+
- Supported OS: Linux, macOS, Windows

## Usage

### Basic Usage

```bash
readchop -i input.fastq -d pattern.db -p pattern_list.txt -o output_dir
```

### Main Parameters

| Parameter | Short | Description | Default |
|-----------|-------|-------------|---------|
| `--inputs` | `-i` | Input file path | Required |
| `--outdir` | `-o` | Output directory name | outdir |
| `--threads` | `-t` | Number of threads | 20 |
| `--min-length` | `-m` | Minimum sequence length filter threshold | 100 |
| `--pattern-files` | `-p` | Pattern file list | Required |
| `--db` | `-d` | Pattern database file | Required |
| `--window-size` | `-w` | Search window size <left,right> | 400,400 |
| `--pattern-error-rate` | `-e` | Pattern matching error rate <left,right> | 0.2,0.2 |
| `--match` | | Pattern matching type: single/dual | single |

### Advanced Parameters

| Parameter | Description | Default |
|-----------|-------------|---------|
| `--trim-mode` | Sequence trimming mode: 0=trim all, 1=keep one pattern, 2=keep two patterns... | 0 |
| `--write-type` | Write type: names=use names, type=use type | type |
| `--pos` | Whether to use position information for more precise detection | false |
| `--shift` | Position offset for multi-pattern splitting | 3 |
| `--maxdist` | Maximum distance threshold | 4 |
| `--id_sep` | Record ID separator | % |

## Usage Examples

### 1. Basic Demultiplexing

```bash
# Demultiplex using single pattern matching
readchop \
    -i example/example.fastq \
    -d example/ont_bc_pattern.db \
    -p example/ont_bc_index.list \
    -o output_dir \
    -t 8
```

### 2. Dual Pattern Matching

```bash
# Use dual pattern matching
readchop \
    -i input.fastq \
    -d pattern.db \
    -p pattern_list.txt \
    -o output_dir \
    --match dual \
    -w 100,100 \
    -e 0.3,0.3
```

### 3. Preview Splitting Results

```bash
# Preview splitting results (with color highlighting)
readchop view \
    -i example/example.fastq \
    -d example/ont_bc_pattern.db \
    -p example/ont_bc_index.list | less
```

### 4. High-Performance Processing

```bash
# Use multi-threading and optimized parameters
readchop \
    -i large_dataset.fastq \
    -d barcode_patterns.db \
    -p barcode_index.list \
    -o results \
    -t 16 \
    -w 200,200 \
    -e 0.2,0.2 \
    --match single
```

### 5. Database Encryption

```bash
# Encrypt pattern database file
readchop encrypt pattern_database.db
```

## Performance Benchmarking

ReadChop performs excellently in performance tests, supporting multi-threaded parallel processing:

```bash
# Benchmark example (from benchmark_simplified.sh)
hyperfine \
    --runs 3 \
    --warmup 1 \
    --parameter-list threads 1,2,4,8,16 \
    --export-json readchop_multithread.json \
    --export-markdown readchop_multithread.md \
    --command-name "ReadChop-{threads}threads" \
    --cleanup "rm -rf output_{threads}" \
    --output null \
    "readchop -i input.fastq -w 100,100 -e 0.3,0.3 --match single -p pattern.list -d pattern.db -o output_{threads} -t {threads}"
```

## Input File Formats

### Pattern File Format (pattern_list.txt)

```text
#index_F	index_R	type
BC01	BC01	ONT-BC01
BC02	BC02	ONT-BC02
BC03	BC03	ONT-BC03
```

### FASTQ Input Format

Supports standard FASTQ format, including compressed .gz files.

## Output Files

ReadChop creates the following files in the specified output directory:

- FASTQ files classified by barcode
- Unmatched sequence files
- Processing statistics

## Subcommands

### view - Preview Function

```bash
readchop view -i input.fastq -d pattern.db -p pattern_list.txt
```

### encrypt - Database Encryption

```bash
readchop encrypt pattern_database.db
```

## Performance Optimization Tips

1. **Thread Count**: Set appropriate thread count based on CPU cores
2. **Window Size**: Adjust search window size based on barcode length
3. **Error Rate**: Adjust matching error rate based on data quality
4. **Memory Usage**: Monitor memory usage when processing large files

## Troubleshooting

### Common Issues

1. **Insufficient Memory**: Reduce thread count or use smaller window size
2. **Low Match Rate**: Check pattern file format and error rate settings
3. **Slow Processing**: Increase thread count or optimize parameter settings

### Logging Information

ReadChop provides detailed logging information to help diagnose issues:

```bash
# Set log level
export RUST_LOG=debug
readchop -i input.fastq -d pattern.db -p pattern_list.txt -o output
```

## License

This project is licensed under an open source license. See LICENSE file for details.

## Author

- Author: jiangchen
- Email: cherryamme@qq.com
- Version: 0.0.1
- Release Date: 2025-09-18

## Contributing

Issues and Pull Requests are welcome to improve ReadChop.

## Changelog

### v0.0.1 (2025-09-18)
- Initial release
- Support for basic barcode demultiplexing functionality
- Support for multi-threading
- Support for preview and encryption features