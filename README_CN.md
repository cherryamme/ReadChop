# ReadChop

<div align="center">
  <img src="assets/readchop.svg" alt="ReadChop Logo" width="400"/>
  
  <h3>⚡ 高效的三代测序数据拆分工具</h3>
  
  <p>基于模式匹配的长读段 FASTQ/GZ 文件拆分工具，支持多线程并行处理</p>
  
  <!-- 项目徽章 -->
  <p>
    <img src="https://img.shields.io/badge/Rust-1.70+-orange?style=flat-square&logo=rust" alt="Rust Version"/>
    <img src="https://img.shields.io/badge/License-MIT-green?style=flat-square" alt="License"/>
    <img src="https://img.shields.io/badge/Platform-Linux%20%7C%20macOS%20%7C%20Windows-blue?style=flat-square" alt="Platform"/>
    <img src="https://img.shields.io/badge/Status-Active-brightgreen?style=flat-square" alt="Status"/>
  </p>
</div>

## 🚀 项目简介

ReadChop 是一个专为三代测序数据设计的命令行工具，用于基于指定模式拆分长读段 FASTQ/GZ 文件。该工具支持多线程并行处理，提供高效的序列拆分和条形码识别功能。

## ✨ 核心特性

<table>
<tr>
<td width="50%">

### 🔧 核心功能
- **🎯 高效拆分**: 基于条形码模式的序列拆分
- **⚡ 多线程处理**: 支持多线程并行处理，提升处理速度
- **🔍 灵活匹配**: 支持单模式和双模式匹配
- **🛡️ 容错机制**: 可配置的匹配错误率
- **👁️ 预览功能**: 支持拆分结果预览，带颜色高亮
- **🔐 数据库加密**: 支持模式数据库文件加密

</td>
<td width="50%">

### 🚀 性能优势
- **⚡ 高速处理**: 优化的算法实现
- **💾 内存高效**: 低内存占用设计
- **🔧 易于配置**: 丰富的参数选项
- **📊 详细统计**: 完整的处理统计信息
- **🎨 可视化**: 彩色预览功能
- **🔒 安全可靠**: 数据加密保护

</td>
</tr>
</table>

## 📦 安装指南

### 🔨 从源码构建

```bash
# 克隆仓库
git clone https://github.com/cherryamme/ReadChop.git
cd ReadChop

# 构建发布版本
cargo build --release

# 可执行文件位于 target/release/readchop
```

### 📋 系统要求

| 组件 | 要求 |
|------|------|
| **Rust** | 1.70+ |
| **操作系统** | Linux, macOS, Windows |
| **内存** | 建议 4GB+ |
| **存储** | 根据数据大小而定 |

### 🚀 快速开始

```bash
# 检查安装
./target/release/readchop --version

# 运行示例
./target/release/readchop -i example/example.fastq -d example/ont_bc_pattern.db -p example/ont_bc_index.list -o output_dir
```

## 🎯 使用指南

### 💡 基本用法

```bash
readchop -i input.fastq -d pattern.db -p pattern_list.txt -o output_dir
```

### 📋 主要参数

<div align="center">

| 参数 | 简写 | 描述 | 默认值 |
|------|------|------|--------|
| `--inputs` | `-i` | 输入文件路径 | **必需** |
| `--outdir` | `-o` | 输出目录名称 | `outdir` |
| `--threads` | `-t` | 线程数量 | `20` |
| `--min-length` | `-m` | 最小序列长度阈值 | `100` |
| `--pattern-files` | `-p` | 模式文件列表 | **必需** |
| `--db` | `-d` | 模式数据库文件 | **必需** |
| `--window-size` | `-w` | 搜索窗口大小 <左,右> | `400,400` |
| `--pattern-error-rate` | `-e` | 模式匹配错误率 <左,右> | `0.2,0.2` |
| `--match` | | 模式匹配类型: single/dual | `single` |

</div>

### 🔧 高级参数

<div align="center">

| 参数 | 描述 | 默认值 |
|------|------|--------|
| `--trim-mode` | 序列修剪模式: 0=全部修剪, 1=保留一个模式, 2=保留两个模式... | `0` |
| `--write-type` | 写入类型: names=使用名称, type=使用类型 | `type` |
| `--pos` | 是否使用位置信息进行更精确的检测 | `false` |
| `--shift` | 多模式拆分的位置偏移 | `3` |
| `--maxdist` | 最大距离阈值 | `4` |
| `--id_sep` | 记录ID分隔符 | `%` |

</div>

## 🚀 使用示例

### 1️⃣ 基础拆分

```bash
# 使用单模式匹配进行拆分
readchop \
    -i example/example.fastq \
    -d example/ont_bc_pattern.db \
    -p example/ont_bc_index.list \
    -o output_dir \
    -t 8
```

### 2️⃣ 双模式匹配

```bash
# 使用双模式匹配
readchop \
    -i input.fastq \
    -d pattern.db \
    -p pattern_list.txt \
    -o output_dir \
    --match dual \
    -w 100,100 \
    -e 0.3,0.3
```

### 3️⃣ 预览拆分结果

```bash
# 预览拆分结果（带颜色高亮）
readchop view \
    -i example/example.fastq \
    -d example/ont_bc_pattern.db \
    -p example/ont_bc_index.list | less
```

### 4️⃣ 高性能处理

```bash
# 使用多线程和优化参数
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

### 5️⃣ 数据库加密

```bash
# 加密模式数据库文件
readchop encrypt pattern_database.db
```

## 📊 性能基准测试

ReadChop 在性能测试中表现优异，支持多线程并行处理：

```bash
# 基准测试示例 (来自 benchmark_simplified.sh)
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

### 🏆 性能特点

- **⚡ 多线程加速**: 支持多核并行处理
- **💾 内存优化**: 高效的内存使用策略
- **🔧 参数调优**: 丰富的性能调优选项
- **📈 线性扩展**: 线程数与性能呈线性关系

## 📁 文件格式

### 📋 模式文件格式 (pattern_list.txt)

```text
#index_F	index_R	type
BC01	BC01	ONT-BC01
BC02	BC02	ONT-BC02
BC03	BC03	ONT-BC03
```

### 📄 FASTQ 输入格式

支持标准 FASTQ 格式，包括压缩的 .gz 文件。

### 📤 输出文件

ReadChop 在指定的输出目录中创建以下文件：

- **📊 按条形码分类的 FASTQ 文件**
- **❌ 未匹配的序列文件**
- **📈 处理统计信息**

## 🔧 子命令

### 👁️ view - 预览功能

```bash
readchop view -i input.fastq -d pattern.db -p pattern_list.txt
```

### 🔐 encrypt - 数据库加密

```bash
readchop encrypt pattern_database.db
```

## ⚡ 性能优化建议

<div align="center">

| 优化项 | 建议 | 说明 |
|--------|------|------|
| **🧵 线程数** | 根据 CPU 核心数设置 | 建议设置为 CPU 核心数的 1-2 倍 |
| **🪟 窗口大小** | 根据条形码长度调整 | 条形码越长，窗口越大 |
| **❌ 错误率** | 根据数据质量调整 | 高质量数据可降低错误率 |
| **💾 内存使用** | 监控大文件处理时的内存 | 必要时减少线程数 |

</div>

## 🔧 故障排除

### ❗ 常见问题

<table>
<tr>
<td width="50%">

#### 🚨 问题诊断
- **💾 内存不足**: 减少线程数或使用更小的窗口大小
- **📉 匹配率低**: 检查模式文件格式和错误率设置
- **🐌 处理缓慢**: 增加线程数或优化参数设置

</td>
<td width="50%">

#### 🔍 调试信息
- **📊 详细日志**: 使用 `RUST_LOG=debug` 获取详细日志
- **📈 性能监控**: 监控 CPU 和内存使用情况
- **🔧 参数调优**: 根据数据特点调整参数

</td>
</tr>
</table>

### 📝 日志信息

ReadChop 提供详细的日志信息来帮助诊断问题：

```bash
# 设置日志级别
export RUST_LOG=debug
readchop -i input.fastq -d pattern.db -p pattern_list.txt -o output
```

## 📄 许可证

本项目采用开源许可证。详情请参见 [LICENSE](LICENSE) 文件。

## 👨‍💻 作者信息

<div align="center">

| 信息 | 详情 |
|------|------|
| **👤 作者** | jiangchen |
| **📧 邮箱** | cherryamme@qq.com |
| **🏷️ 版本** | 0.0.1 |
| **📅 发布日期** | 2025-09-18 |

</div>

## 🤝 贡献指南

我们欢迎各种形式的贡献来改进 ReadChop！

### 🚀 如何贡献

- **🐛 报告问题**: 在 Issues 中报告 bug 或提出功能请求
- **💡 提出建议**: 分享您的想法和改进建议
- **🔧 提交代码**: 通过 Pull Request 提交代码改进
- **📖 完善文档**: 帮助改进文档和示例

### 📋 贡献流程

1. Fork 本仓库
2. 创建功能分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 打开 Pull Request

## 📝 更新日志

### 🎉 v0.0.1 (2025-09-18)

- **🎯 初始发布**
- **🔧 基础功能**: 支持基本的条形码拆分功能
- **⚡ 多线程**: 支持多线程并行处理
- **👁️ 预览功能**: 支持拆分结果预览
- **🔐 加密功能**: 支持模式数据库加密

---

<div align="center">

**⭐ 如果这个项目对您有帮助，请给我们一个星标！**

[![GitHub stars](https://img.shields.io/github/stars/cherryamme/ReadChop?style=social)](https://github.com/cherryamme/ReadChop)
[![GitHub forks](https://img.shields.io/github/forks/cherryamme/ReadChop?style=social)](https://github.com/cherryamme/ReadChop)

</div>