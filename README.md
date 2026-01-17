# MIPS Simulator

A simple MIPS simulator written in Rust.

It's a work in progress, can't really do much yet.

It can only execute the simple examples provided with optional debugging output.

When it's done, it will be compatible with [Mars](https://github.com/dpetersanderson/MARS).

## Usage

```bash
cargo run -- examples/hello_world.asm
```

## Options

```bash
-h, --help           Print the help message
-a, --args           Print the arguments
-t, --tokens         Print the tokens
-i, --instructions   Print the instructions
-m, --memory         Print the memory
-v, --version        Print program version
```

## License

This project is licensed under the UNLICENSE License.
