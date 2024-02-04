# Level 6 diskette image utility

`l6disk` is a simple utility that facilitates the creation of Level6-formatted HFE floppy images, to be written to a real disk with a [Greaseweazle](https://github.com/keirf/greaseweazle) or a [FlashFloppy](https://github.com/keirf/flashfloppy)-flashed Gotek floppy emulator.

## Installation

### Building from source

Build the project with `cargo`

```bash
cargo build --release
```

Install the resulting binary

```
cargo install --path .
```

## Usage

```
Level6 diskette image utility

Usage: l6disk [OPTIONS] <INPUT> <OUTPUT>

Arguments:
  <INPUT>   Input data disk image
  <OUTPUT>  Output raw disk image

Options:
  -l, --ignore-errors              Ignore image conversion errors
  -p, --disk-format <DISK_FORMAT>  Disk format preset [default: level6] [possible values: level6, ibm8dssd]
  -c, --cylinders <CYLINDERS>      Number of cylinders
  -e, --heads <HEADS>              Number of heads (sides)
  -s, --sectors <SECTORS>          Number of Sectors per track
  -b, --sector-size <SECTOR_SIZE>  Sector size
  -r, --cell-rate <CELL_RATE>      Cell rate (kBps)
  -m, --spindle-rpm <SPINDLE_RPM>  Spindle RPM
  -i, --interleave <INTERLEAVE>    Disk sector interleave
  -h, --help                       Print help
  -V, --version                    Print version
```

NOTE: option `-l, --ignore-errors` makes `l6disk` ignore sector division errors (when the number of bytes in the input image cannot be evenly divided into sectors) and sector alignment errors (when the number of sectors provided in the input image is different than the number of sectors of the disk), either truncating the image or filling the remaining part of the disk with `0x00`.

### Usage examples

#### Convert a `.img` image to Level6-formatted `.hfe` image

```bash
l6disk input.img output.hfe
```

#### Create a blank Level6-formatted `.hfe` image

```bash
dd if=/dev/zero of=tmp.img bs=128 count=2002
l6disk tmp.img output.hfe
```

Note the `128 byte` sector size and `2002` sectors.

## Level6 diskette format

The Honeywell Level6 uses standard 8-inch SS/SD floppy disks.

The track format is standard IBM3470-style FM (see [DP8473 floppy controller datasheet](https://www.wdj-consulting.com/blog/floppy-lit/DS009384.PDF) for more info )

#### Specifications

| Specification     | Value    |
| ----------------- | -------- |
| Tracks            | 77       |
| Sides             | 1        |
| Sectors per track | 26       |
| Bytes per sector  | 128      |
| Clock rate        | 500 kbps |
| Data rate         | 250 kbps |
| Spindle RPM       | 360      |
