# ReadChop

<div align="center">
  <img src="assets/readchop.svg" alt="ReadChop Logo" width="400"/>
  
  <h3>⚡ High-Performance Third-Generation Sequencing Data Demultiplexing Tool</h3>
  
  <p>Pattern-based long-read FASTQ/GZ file demultiplexing tool with multi-threaded parallel processing support</p>
  
  <!-- Project Badges -->
  <p>
    <img src="https://img.shields.io/badge/Rust-1.70+-orange?style=flat-square&logo=rust" alt="Rust Version"/>
    <img src="https://img.shields.io/badge/License-MIT-green?style=flat-square" alt="License"/>
    <img src="https://img.shields.io/badge/Platform-Linux%20%7C%20macOS%20%7C%20Windows-blue?style=flat-square" alt="Platform"/>
    <img src="https://img.shields.io/badge/Status-Active-brightgreen?style=flat-square" alt="Status"/>
  </p>
  
  <!-- Language Links -->
  <p>
    <a href="README_CN.md">中文版</a> | <strong>English</strong>
  </p>
</div>

## 🚀 Project Overview

ReadChop is a command-line tool specifically designed for third-generation sequencing data, used to demultiplex long-read FASTQ/GZ files based on specified patterns. The tool supports multi-threaded parallel processing, providing efficient sequence demultiplexing and barcode identification capabilities.

## ✨ Core Features

<table>
<tr>
<td width="50%">

### 🔧 Core Functionality
- **🎯 Efficient Demultiplexing**: Pattern-based sequence demultiplexing
- **⚡ Multi-threaded Processing**: Multi-threaded parallel processing support
- **🔍 Flexible Matching**: Single and dual pattern matching support
- **🛡️ Error Tolerance**: Configurable matching error rates
- **👁️ Preview Function**: Demultiplexing result preview with color highlighting
- **🔐 Database Encryption**: Pattern database file encryption support

</td>
<td width="50%">

### 🚀 Performance Advantages
- **⚡ High-Speed Processing**: Optimized algorithm implementation
- **💾 Memory Efficient**: Low memory footprint design
- **🔧 Easy Configuration**: Rich parameter options
- **📊 Detailed Statistics**: Complete processing statistics
- **🎨 Visualization**: Color preview functionality
- **🔒 Security**: Data encryption protection

</td>
</tr>
</table>

## 📦 Installation Guide

### 🔨 Build from Source

```bash
# Clone repository
git clone https://github.com/cherryamme/ReadChop.git
cd ReadChop

# Build release version
cargo build --release

# Executable located at target/release/readchop
```

### 📋 System Requirements

| Component | Requirements |
|-----------|-------------|
| **Rust** | 1.70+ |
| **Operating System** | Linux, macOS, Windows |
| **Memory** | Recommended 4GB+ |
| **Storage** | Depends on data size |

### 🚀 Quick Start

```bash
# Check installation
./target/release/readchop --version

# Run example
./target/release/readchop -i example/example.fastq -d example/ont_bc_pattern.db -p example/ont_bc_index.list -o output_dir
```

## 🎯 Usage Guide

### 💡 Basic Usage

```bash
readchop -i input.fastq -d pattern.db -p pattern_list.txt -o output_dir
```

### 📋 Main Parameters

<div align="center">

| Parameter | Short | Description | Default |
|-----------|-------|-------------|---------|
| `--inputs` | `-i` | Input file paths | **Required** |
| `--outdir` | `-o` | Output directory name | `outdir` |
| `--threads` | `-t` | Number of threads | `20` |
| `--min-length` | `-m` | Minimum sequence length threshold | `100` |
| `--pattern-files` | `-p` | Pattern file list | **Required** |
| `--db` | `-d` | Pattern database file | **Required** |
| `--window-size` | `-w` | Search window size <left,right> | `400,400` |
| `--pattern-error-rate` | `-e` | Pattern matching error rate <left,right> | `0.2,0.2` |
| `--match` | | Pattern matching type: single/dual | `single` |

</div>

### 🔧 Advanced Parameters

<div align="center">

| Parameter | Description | Default |
|-----------|-------------|---------|
| `--trim-mode` | Sequence trimming mode: 0=trim all, 1=keep one pattern, 2=keep two patterns... | `0` |
| `--write-type` | Write type: names=use names, type=use type | `type` |
| `--pos` | Use position information for more precise detection | `false` |
| `--shift` | Position offset for multi-pattern demultiplexing | `3` |
| `--maxdist` | Maximum distance threshold | `4` |
| `--id_sep` | Record ID separator | `%` |

</div>

## 🚀 Usage Examples

### 1️⃣ Basic Demultiplexing

```bash
# Demultiplex using single pattern matching
readchop \
    -i example/example.fastq \
    -d example/ont_bc_pattern.db \
    -p example/ont_bc_index.list \
    -o output_dir \
    -t 8
```

### 2️⃣ Dual Pattern Matching

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

### 3️⃣ Preview Demultiplexing Results

```bash
# Preview demultiplexing results (with color highlighting)
readchop view \
    -i example/example.fastq \
    -d example/ont_bc_pattern.db \
    -p example/ont_bc_index.list | less
```

### 4️⃣ High-Performance Processing

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

### 5️⃣ Database Encryption

```bash
# Encrypt pattern database file
readchop encrypt pattern_database.db
```

## 📊 Performance Benchmarks

ReadChop demonstrates excellent performance in testing, supporting multi-threaded parallel processing:

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

### 🏆 Performance Characteristics

- **⚡ Multi-threaded Acceleration**: Multi-core parallel processing support
- **💾 Memory Optimization**: Efficient memory usage strategies
- **🔧 Parameter Tuning**: Rich performance tuning options
- **📈 Linear Scaling**: Linear relationship between thread count and performance

## 📁 File Formats

### 📋 Pattern File Format (pattern_list.txt)

```text
#index_F	index_R	type
BC01	BC01	ONT-BC01
BC02	BC02	ONT-BC02
BC03	BC03	ONT-BC03
```

### 📄 FASTQ Input Format

Supports standard FASTQ format, including compressed .gz files.

### 📤 Output Files

ReadChop creates the following files in the specified output directory:

- **📊 Barcode-classified FASTQ files**
- **❌ Unmatched sequence files**
- **📈 Processing statistics**

## 🔧 Subcommands

### 👁️ view - Preview Function

```bash
readchop view -i input.fastq -d pattern.db -p pattern_list.txt
```

### 🔐 encrypt - Database Encryption

```bash
readchop encrypt pattern_database.db
```

## ⚡ Performance Optimization Recommendations

<div align="center">

| Optimization | Recommendation | Description |
|--------------|----------------|-------------|
| **🧵 Thread Count** | Set based on CPU cores | Recommended 1-2x CPU core count |
| **🪟 Window Size** | Adjust based on barcode length | Larger window for longer barcodes |
| **❌ Error Rate** | Adjust based on data quality | Lower error rate for high-quality data |
| **💾 Memory Usage** | Monitor memory during large file processing | Reduce thread count if necessary |

</div>

## 🔧 Troubleshooting

### ❗ Common Issues

<table>
<tr>
<td width="50%">

#### 🚨 Problem Diagnosis
- **💾 Insufficient Memory**: Reduce thread count or use smaller window size
- **📉 Low Match Rate**: Check pattern file format and error rate settings
- **🐌 Slow Processing**: Increase thread count or optimize parameter settings

</td>
<td width="50%">

#### 🔍 Debug Information
- **📊 Detailed Logs**: Use `RUST_LOG=debug` for detailed logs
- **📈 Performance Monitoring**: Monitor CPU and memory usage
- **🔧 Parameter Tuning**: Adjust parameters based on data characteristics

</td>
</tr>
</table>

### 📝 Log Information

ReadChop provides detailed log information to help diagnose issues:

```bash
# Set log level
export RUST_LOG=debug
readchop -i input.fastq -d pattern.db -p pattern_list.txt -o output
```

## 📄 License

This project is licensed under an open source license. See the [LICENSE](LICENSE) file for details.

## 👨‍💻 Author Information

<div align="center">

| Information | Details |
|-------------|---------|
| **👤 Author** | jiangchen |
| **📧 Email** | cherryamme@qq.com |
| **🏷️ Version** | 0.0.1 |
| **📅 Release Date** | 2025-09-18 |

</div>

## 🤝 Contributing

We welcome contributions of all kinds to improve ReadChop!

### 🚀 How to Contribute

- **🐛 Report Issues**: Report bugs or request features in Issues
- **💡 Suggest Ideas**: Share your ideas and improvement suggestions
- **🔧 Submit Code**: Submit code improvements via Pull Request
- **📖 Improve Documentation**: Help improve documentation and examples

### 📋 Contribution Process

1. Fork this repository
2. Create a feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## 📝 Changelog

### 🎉 v0.0.1 (2025-09-18)

- **🎯 Initial Release**
- **🔧 Basic Features**: Support for basic barcode demultiplexing functionality
- **⚡ Multi-threading**: Multi-threaded parallel processing support
- **👁️ Preview Function**: Demultiplexing result preview support
- **🔐 Encryption Function**: Pattern database encryption support

---

<div align="center">

**⭐ If this project helps you, please give us a star!**

[![GitHub stars](https://img.shields.io/github/stars/cherryamme/ReadChop?style=social)](https://github.com/cherryamme/ReadChop)
[![GitHub forks](https://img.shields.io/github/forks/cherryamme/ReadChop?style=social)](https://github.com/cherryamme/ReadChop)

</div>