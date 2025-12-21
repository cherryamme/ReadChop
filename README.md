# ReadChop

A high-performance command-line tool for demultiplexing third-generation sequencing long-read FASTQ/GZ files based on specified patterns.

## Overview

ReadChop is designed specifically for third-generation sequencing data, used to split long-read FASTQ/GZ files based on specified patterns. It supports multi-threaded parallel processing, providing efficient sequence demultiplexing and barcode identification capabilities.

**Version:** 1.0.0 
**Author:** jiangchen  
**Email:** cherryamme@qq.com  
**Release Date:** 2025-09-18

## Features

- **Pattern-based demultiplexing**: Split sequences based on barcode patterns
- **Multi-threaded processing**: Parallel processing support for high performance
- **Flexible matching**: Support for single and dual pattern matching
- **Error tolerance**: Configurable matching error rates
- **Preview mode**: Preview demultiplexing results with color highlighting
- **Database encryption**: Encrypt pattern database files for security
- **Fusion detection**: Optional fusion sequence detection

## Installation

### Build from Source

```bash
# Clone repository
git clone https://github.com/cherryamme/ReadChop.git
cd ReadChop

# Build release version
cargo build --release

# Executable located at target/release/readchop
```

### System Requirements

- **Rust**: 1.70+
- **Operating System**: Linux, macOS, Windows
- **Memory**: Recommended 4GB+
- **Threads**: Must be greater than 2 (default: 20)

## Quick Start

```bash
# Check installation
./target/release/readchop --version

# Run with example data (see example/ folder for test data)
./target/release/readchop \
    -i example/example.fastq \
    -d example/ont_bc_pattern.db \
    -p example/ont_bc_index.list \
    -o output_dir
```

**Note:** The `example/` folder contains test data files for demonstration purposes.

## Usage

### Basic Command

```bash
readchop [OPTIONS] --pattern-files <PATTERN_FILES>... --db <PATTERN_DB_FILE>
```

### Required Options

| Option | Short | Description |
|--------|-------|-------------|
| `--pattern-files` | `-p` | Pattern file list (one or more files) |
| `--db` | `-d` | Pattern database file |

### Common Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--inputs` | `-i` | Input file paths (one or more files) | - |
| `--outdir` | `-o` | Output directory name | `outdir` |
| `--threads` | `-t` | Number of threads (must be > 2) | `20` |
| `--min-length` | `-m` | Minimum sequence length filter threshold | `100` |
| `--window-size` | `-w` | Search window size `<left,right>` | `400,400` |
| `--pattern-error-rate` | `-e` | Pattern matching error rate `<left,right>` (0-0.5) | `0.2,0.2` |
| `--match` | | Pattern matching type: `single` or `dual` | `single` |
| `--trim-mode` | | Sequence trimming mode: 0=trim all, 1=keep one pattern, 2=keep two patterns... | `0` |
| `--write-type` | | Write type: `names` (use names) or `type` (use types) | `type` |
| `--pos` | | Use position information for more precise detection | `false` |
| `--shift` | | Position offset for multi-pattern splitting | `3` |
| `--maxdist` | | Maximum distance threshold | `4` |
| `--id_sep` | | Record ID separator | `%` |
| `--fusion` | `-f` | Fusion detection file | - |
| `--fe` | | Fusion detection error rate | `0.2` |
| `--write_all` | | Write all reads including unknown sequences | `false` |
| `--enable_logger` | | Enable logger to write reads_log.gz file | `false` |
| `--uncompress` | | Disable gzip compression for output files | `false` |
| `--num` | `-n` | Log recording interval | `500000` |

## Commands

### view - Preview Results

Preview barcode detection results with color highlighting:

```bash
readchop view -i input.fastq -d pattern.db -p pattern_list.txt | less
```

### encrypt - Encrypt Database

Encrypt pattern database file:

```bash
readchop encrypt pattern_database.db
```

## Examples

### Example 1: Basic Demultiplexing

```bash
readchop \
    -i example/example.fastq \
    -d example/ont_bc_pattern.db \
    -p example/ont_bc_index.list \
    -o output_dir \
    -t 8
```

### Example 2: Dual Pattern Matching

```bash
readchop \
    -i input.fastq \
    -d pattern.db \
    -p pattern_list.txt \
    -o output_dir \
    --match dual \
    -w 100,100 \
    -e 0.3,0.3
```

### Example 3: Preview Mode

```bash
readchop view \
    -i example/example.fastq \
    -d example/ont_bc_pattern.db \
    -p example/ont_bc_index.list | less
```


## File Formats

### Pattern File Format

The pattern file should contain tab-separated values with the following format:

```text
#index_F	index_R	type
BC01	BC01	ONT-BC01
BC02	BC02	ONT-BC02
BC03	BC03	ONT-BC03
```

### Input Format

- Supports standard FASTQ format
- Supports compressed `.gz` files

### Output Files

ReadChop creates the following files in the specified output directory:

- Barcode-classified FASTQ files (compressed by default)
- Unmatched sequence files (if `--write_all` is enabled)
- Processing statistics files
- Log file (if `--enable_logger` is enabled)

## Test Data

The `example/` folder contains test data files for testing and demonstration:

- `example.fastq` - Sample FASTQ file
- `example.fasta` - Sample FASTA file
- `ont_bc_index.list` - Pattern index list file
- `ont_bc_pattern.db` - Pattern database file




## License

This project is licensed under an open source license. See the [LICENSE](LICENSE) file for details.