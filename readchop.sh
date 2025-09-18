### 正常拆分
readchop \
    -i example/example.fastq \
    -d example/ont_bc_pattern.db \
    -p example/ont_bc_index.list \
    -o test_out

###

### 预览拆分
readchop view\
    -i example/example.fastq \
    -d example/ont_bc_pattern.db \
    -p example/ont_bc_index.list |less

### 