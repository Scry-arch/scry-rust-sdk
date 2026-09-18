# Board profiles

A board profile tells the SDK how to build for and talk to one piece of Scry hardware. Profiles are
TOML files. The ones in this directory ship with the SDK and are selected by name
(`--board scry5-nexys-a7`); a project can also use its own with `--board path/to/board.toml`.

## Profile format

```toml
# Shown in messages.
name = "Scry-5 (Nexys A7)"

[link]
# Extra linker arguments. Optional.
args = ["--image-base=0x0", "-n"]

[memory]
# The half-open address range that a program's code and data, including zero-initialised data,
# must fit in. Required.
program = { start = 0x0000_0000, end = 0x0001_0000 }

[loader]
# Serial speed. Optional, 115200 by default.
baud = 115200
# Largest payload the loader accepts, in bytes. Optional, unlimited by default.
max-payload = 0x0010_0000
```

Unknown keys are rejected, so a typo is an error rather than a silently ignored setting.

Two linker arguments matter for a board without an MMU:

- `--image-base=<address>` places the program. The ELF headers sit at that address, followed by the
  read-only data, the code and the writable data. The headers are loaded along with everything
  else; they are harmless, because execution starts at the ELF's entry point, not at the first
  byte.
- `-n` stops the linker from padding each segment out to a page boundary.

## What `scry-load` does with an ELF

`cargo scry run --board <board>` runs `scry-load` with the freshly linked ELF. It

1. reads the ELF's loadable segments and its entry point,
2. checks that everything the program occupies, zero-initialised data included, lies inside
   `memory.program`, that the entry point is inside the image, and that the image is within
   `max-payload`,
3. flattens the segments into one block starting at the lowest address, with zeros in the gaps and
   without the zero-initialised tail,
4. verifies that block against the ELF byte for byte,
5. sends it, then shows the program's output and finally its returned operands in scryer's format.

Options go after `--`: `--check` stops after step 2, `--image <file>` writes the block to a file
instead of sending it, `--port <name>` picks the serial port, `--no-wait` exits after launch.

## Loader protocol

This is what a board's boot loader has to implement. All integers are little-endian.

Download frame, host to board:

| bytes | content |
|---|---|
| 8 | the magic `SCRYLD! `, with its trailing space |
| 4 | entry address: where execution starts |
| 4 | load address: where the first payload byte goes |
| 4 | payload length |
| n | payload |
| 1 | checksum: the sum of the payload bytes, modulo 256 |

The loader writes the payload to memory at the load address, clears the rest of the program memory
after it (programs rely on this for their zero-initialised data), starts execution at the entry
address, and answers with one byte: `K` launched, `E` bad checksum, `T` timed out mid-frame.

Report frame, board to host, when the program ends: the magic `SCRYRT! `, a status byte (0 for a
clean halt, anything else for a trap), an operand count, then per returned operand a tag byte and a
32-bit value. In the tag, the low two bits give the operand's width as `8 << n` bits, `0x20` marks
it signed, `0x40` not-a-number and `0x80` not-a-result. Everything the board sends before the magic
is the program's own output.

The report frame is the one the Scry-5 loader already sends. The download frame replaces its
original one, which used the same magic but was followed directly by a 16-bit length, carried no
addresses, and always started execution at address 0. The two layouts are not compatible: a loader
understands one or the other.
