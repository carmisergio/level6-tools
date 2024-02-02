## File header

| Offset | Length | HXCEMU value           | Description           |
| ------ | ------ | ---------------------- | --------------------- |
| 0x00   | 8      | "HXCPICFE"             | File signature        |
| 0x08   | 1      | 0x00 (0)               | Revision              |
| 0x09   | 1      | 0x4D (77)              | Number of tracks      |
| 0x0A   | 1      | 0x01 (1)               | Number of sides       |
| 0x0B   | 1      | 0x02 (ISOIBM_FM)       | Track encoding        |
| 0x0C   | 2      | 0xFA, 0x00 (250)       | Bit (cell) rate       |
| 0x0E   | 2      | 0x00, 0x00 (0)         | RPM                   |
| 0x10   | 1      | 0x07 (GENERIC_SHUGART) | Floppy interface mode |
| 0x11   | 1      | 0x01 (1)               | DNU                   |
| 0x12   | 2      | 0x01, 0x00 (1)         | Track list offset     |
| 0x14   | 1      | 0xFF (255)             | Write allowed         |
| 0x15   | 1      | 0xFF (255)             | Single step           |
| 0x16   | 1      | 0xFF (255)             | Track0s0 altencoding  |
| 0x17   | 1      | 0xFF (255)             | Track0s0 encoding     |
| 0x18   | 1      | 0xFF (255)             | Track0s1 altencoding  |
| 0x19   | 1      | 0xFF (255)             | Track0s1 encoding     |
