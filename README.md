# chip8-rs

A Chip-8 Interpreter written in Rust that implements all original Chip-8 opcodes. Compiles on Linux. Depends on SDL.

[![Alt text](https://img.youtube.com/vi/44UpUbu2Z9U/0.jpg)](https://www.youtube.com/watch?v=44UpUbu2Z9U)

## Usage

```
Usage: chip8 [OPTIONS] --rom-path <FILE>

Options:
  -r, --rom-path <FILE>  The path of the rom to load
  -h, --help             Print help
  -V, --version          Print version
```

## Keyboard Input

Keys are mapped as such:

**Original**

```
1 2 3 C
4 5 6 D
7 8 9 E
A 0 B F
```

**Emulator**

```
1 2 3 4
Q W E R
A S D F
Y X C V
```

It also works with QWERTZ keyboards.

## Test Roms

Here are resources to test and enjoy this Chip-8 interpreter with.

- https://github.com/Timendus/chip8-test-suite
- https://github.com/kripod/chip8-roms
- https://github.com/NinjaWeedle/chip8-test-rom-with-audio.git
