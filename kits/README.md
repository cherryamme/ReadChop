# ONT Barcode Kits for ReadChop

This directory contains pre-configured barcode kit files for demultiplexing Oxford Nanopore Technologies (ONT) sequencing data using [ReadChop](https://github.com/cherryamme/ReadChop).

## Overview

Each subdirectory represents a specific ONT barcode kit and includes configuration files ready for use with ReadChop. These kits support various sample multiplexing needs, from 12-barcode sets to 96-barcode sets.

## Directory Structure

Each barcode kit folder contains the following files:

```
<Kit-Name>/
├── <Kit-Name>_raw.list      # Raw barcode patterns
├── <Kit-Name>_rear.list     # Barcode patterns with rear adapters
└── ont_bc_pattern.db        # Barcode sequence database
```

## File Formats

### `*_raw.list` (Raw Barcode Patterns)

Contains the basic barcode index assignments without adapter sequences. Format:

```
#index_F    index_R    type
BC01        BC01       ONT-BC01
BC02        BC02       ONT-BC02
...
```

Use this file when your sequencing data contains raw barcode sequences at the read ends.

### `*_rear.list` (Rear Adapter Barcode Patterns)

Contains barcode patterns with rear adapter sequences included. Format:

```
#index_F          index_R            type
BC01rear1         BC01rear1          BC01-type1
BC02rear1         BC02rear1          BC02-type1
...
```

Use this file when you need to demultiplex reads that have both front and rear adapters.

### `ont_bc_pattern.db` (Barcode Sequence Database)

Contains the actual barcode sequences for pattern matching. Format:

```
BC01          AAGAAAGTTGTCGGTGTCTTTGTG
BC01rear1     AAGAAAGTTGTCGGTGTCTTTGTGGTTTTCGCATTTATCGTGAAACGCTTTCGCGTTTTTCGTGCGCCGCTTCA
BC02          TCGATTCCGTTTGTAGTCGTCTGT
...
```