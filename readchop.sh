### 正常拆分
rm -rf test_out
readchop \
    -i example/example.fastq \
    -d example/ont_bc_pattern.db \
    -p example/ont_bc_index.list \
    -o test_out

###


### 检查readid
zcat test_out/default/default/ONT-BC01.fq.gz |head -n 1

### 

### 预览拆分
readchop view\
    -i example/example.fastq \
    -d example/ont_bc_pattern.db \
    -p example/ont_bc_index.list |less

### 