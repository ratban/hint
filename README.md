# Hint Compiler (hintc)

A zero-dependency compiler for the Hint programming language that compiles conversational English directly to native Windows x86_64 executables.

## Overview

The Hint programming language allows you to write programs using natural English sentences. The compiler recognizes three main types of statements:

1. **Say**: Outputs a string to the console
   - Example: `Say "Hello, world!".`

2. **Keep**: Stores a number in memory with a named variable
   - Example: `Keep the number 42 in mind as the answer.`

3. **Stop**: Terminates the program execution
   - Example: `Stop the program.`

## Installation

To build and use the Hint compiler, you need:

- Rust toolchain (rustc, cargo)
- NASM assembler (https://www.nasm.us/)
- Microsoft Visual Studio Build Tools (for the linker)

## Building

```bash
cd hintc
cargo build --release
```

The executable will be available at `target/release/hintc.exe`.

## Usage

```bash
hintc program.ht          # Compile to program.exe
hintc program.ht -o out   # Compile to out.exe
hintc --ast program.ht    # Print AST only
hintc --tokens program.ht # Print tokens only
hintc --keep program.ht   # Keep intermediate files
```

## Examples

### Hello World (`hello_world.ht`)
```
Say "Hello, World!".
Stop the program.
```

### Variable Storage (`variables.ht`)
```
Keep the number 42 in mind as the answer.
Say "The answer is 42.".
Stop the program.
```

## Language Specification

The Hint language is case-insensitive for keywords. The grammar supports:

- `Say "[text]".` - Prints text to standard output
- `Keep the number [number] in mind as the [name].` - Stores a number in a variable
- `Stop the program.` - Exits the program with code 0

## Implementation Details

The compiler follows a traditional three-phase approach:

1. **Lexical Analysis**: Converts source text to tokens
2. **Parsing**: Converts tokens to an Abstract Syntax Tree (AST)
3. **Code Generation**: Converts AST to x86_64 assembly code

The generated assembly uses the Windows x64 calling convention and links against Windows API functions for console output.

## Troubleshooting

If you encounter the error "NASM not found", you need to install NASM (Netwide Assembler) from https://www.nasm.us/ and ensure it's in your system PATH.

## Development

The compiler includes unit tests that can be run with:
```bash
cargo test
```

To see the tokenization output:
```bash
hintc --tokens program.ht
```

To see the AST output:
```bash
hintc --ast program.ht
```

To keep intermediate files (.asm and .obj):
```bash
hintc --keep program.ht
```