# Hint Programming Language

**Hint** is a conversational programming language that allows you to write programs using natural English sentences. It compiles directly to native Windows x86_64 executables with zero runtime dependencies.

![Hint Language Logo](./assets/logo.svg)

## Table of Contents

- [Overview](#overview)
- [Quick Start](#quick-start)
- [Language Specification](#language-specification)
  - [Basic Statements](#basic-statements)
  - [Grammar](#grammar)
  - [Language Rules](#language-rules)
  - [Data Types](#data-types)
- [Compiler Usage](#compiler-usage)
  - [Installation](#installation)
  - [Command Line Options](#command-line-options)
  - [Examples](#examples)
- [VS Code Extension](#vs-code-extension)
  - [Features](#features)
  - [Installation](#installation-1)
  - [Configuration](#configuration)
  - [Keyboard Shortcuts](#keyboard-shortcuts)
- [Architecture](#architecture)
  - [Compiler Phases](#compiler-phases)
  - [Assembly Output](#assembly-output)
- [Examples](#examples-1)
  - [Hello World](#hello-world)
  - [Variables](#variables)
  - [Complete Program](#complete-program)
- [Error Handling](#error-handling)
- [Limitations](#limitations)
- [Future Enhancements](#future-enhancements)
- [Troubleshooting](#troubleshooting)
- [Contributing](#contributing)

## Overview

Hint is designed to make programming accessible to everyone by using natural English instead of traditional programming syntax. The language is:

- **Conversational**: Write code that reads like English sentences
- **Native**: Compiles directly to machine code with no runtime overhead
- **Simple**: Minimal syntax to learn
- **Fast**: Zero-cost abstractions with direct Windows API calls
- **Zero-dependency**: No runtime libraries required

### Philosophy

The Hint language follows the philosophy that programming should be as natural as writing instructions in English. By removing syntactic barriers, Hint makes programming more accessible while maintaining the power and performance of native code.

## Quick Start

1. **Install the compiler**:
   ```bash
   cd hintc
   cargo build --release
   ```

2. **Write your first program** (`hello.ht`):
   ```hint
   Say "Hello, World!".
   Stop the program.
   ```

3. **Compile and run**:
   ```bash
   hintc hello.ht
   ./hello.exe
   ```

## Language Specification

### Basic Statements

Hint supports three fundamental statement types:

#### 1. Say Statement

Outputs text to the console.

```hint
Say "Hello, world!".
```

**Syntax**: `Say "[text]".`

- `[text]`: A string literal enclosed in double quotes
- Must end with a period

#### 2. Keep Statement

Stores a number in memory with a named variable.

```hint
Keep the number 42 in mind as the answer.
```

**Syntax**: `Keep the number [number] in mind as the [name].`

- `[number]`: An integer value (positive or negative)
- `[name]`: A variable name (letters and underscores only)
- Must end with a period

#### 3. Stop Statement

Terminates the program execution.

```hint
Stop the program.
```

**Syntax**: `Stop the program.`

- Must end with a period
- Exits with code 0

### Grammar

The complete Hint language grammar in BNF form:

```bnf
<program>       ::= <statement>*

<statement>     ::= <say_statement>
                  | <keep_statement>
                  | <stop_statement>

<say_statement> ::= "Say" <string> "."

<keep_statement>::= "Keep" "the" "number" <number> "in" "mind" "as" "the" <identifier> "."

<stop_statement>::= "Stop" "the" "program" "."

<string>        ::= '"' <character>* '"'
<number>        ::= "-"? <digit>+
<identifier>    ::= <letter> (<letter> | <digit> | "_")*
<character>     ::= any printable character except unescaped quote
<digit>         ::= "0" | "1" | ... | "9"
<letter>        ::= "a" | ... | "z" | "A" | ... | "Z"
```

### Language Rules

1. **Case Insensitivity**: All keywords are case-insensitive
   - `Say`, `SAY`, and `say` are all equivalent
   - Variable names preserve their case

2. **Sentence Structure**: All statements must end with a period (`.`)

3. **String Literals**: Must be enclosed in double quotes (`"`)
   - Escape sequences: `\"`, `\\`, `\n`, `\r`, `\t`

4. **Numbers**: Signed 32-bit integers
   - Range: -2,147,483,648 to 2,147,483,647
   - No floating-point support (yet)

5. **Variable Names**: 
   - Must start with a letter
   - Can contain letters, numbers, and underscores
   - Case-sensitive

6. **Comments**: Currently not supported in the core language
   - VS Code extension supports `//` line comments for syntax highlighting

### Data Types

| Type    | Description              | Size    | Range                           |
|---------|--------------------------|---------|---------------------------------|
| String  | Text in double quotes    | Variable | Any printable characters       |
| Number  | Signed integer           | 64-bit  | -2³¹ to 2³¹-1                  |
| Boolean | Not directly supported   | N/A     | Use 0 for false, non-zero true |

## Compiler Usage

### Installation

#### Prerequisites

- **Rust toolchain**: Install from [rustup.rs](https://rustup.rs/)
- **NASM assembler**: Download from [nasm.us](https://www.nasm.us/)
- **MSVC linker**: Install Visual Studio Build Tools

#### Building from Source

```bash
# Clone or navigate to the compiler directory
cd hintc

# Build in release mode
cargo build --release

# The executable will be at target/release/hintc.exe
```

#### Adding to PATH

For convenience, add the compiler to your system PATH:

```bash
# Windows (PowerShell)
$env:Path += ";C:\path\to\hintc\target\release"

# Or copy to a directory already in PATH
copy target\release\hintc.exe C:\Windows\
```

### Command Line Options

```
Hint Compiler (hintc) v0.1.0
Compiles conversational English (.ht) to native Windows x86_64 executables.

USAGE:
    hintc [OPTIONS] <input.ht>

OPTIONS:
    -o, --output <NAME>    Output executable name (default: input file stem)
    --tokens               Print tokens and exit
    --ast                  Print AST and exit
    --keep                 Keep intermediate .asm and .obj files
    -v, --verbose          Enable verbose output
    --lsp                  Start language server for IDE integration
    -h, --help             Print this help message
    -V, --version          Print version information
```

### Examples

#### Basic Compilation

```bash
# Compile to program.exe
hintc program.ht

# Compile with custom output name
hintc program.ht -o myapp

# Compile with verbose output
hintc program.ht --verbose
```

#### Debugging

```bash
# View tokenized output
hintc --tokens program.ht

# View parsed AST
hintc --ast program.ht

# Keep intermediate files for inspection
hintc --keep program.ht
```

#### Language Server Mode

```bash
# Start LSP server (used by VS Code extension)
hintc --lsp
```

## VS Code Extension

### Features

- **Syntax Highlighting**: Full colorization of Hint source files
- **Language Server Protocol (LSP)**:
  - Autocomplete for statements and variables
  - Go-to-definition for variables
  - Hover documentation
  - Real-time diagnostics
  - Document symbols
- **Integrated Compilation**: Compile with Ctrl+Shift+B
- **Debug Tools**: View tokens and AST directly in VS Code

### Installation

1. **From VSIX** (if published):
   - Download the `.vsix` file
   - In VS Code: Extensions → ⋯ → Install from VSIX

2. **From Source**:
   ```bash
   cd hint-vscode
   
   # Install dependencies
   npm install
   
   # Build extension
   npm run compile
   
   # Run extension (opens new VS Code window)
   # Press F5 in VS Code
   ```

### Configuration

Access via: File → Preferences → Settings → Extensions → Hint

| Setting | Default | Description |
|---------|---------|-------------|
| `hint.compilerPath` | `hintc` | Path to the Hint compiler |
| `hint.enableLSP` | `true` | Enable language server features |
| `hint.verboseOutput` | `false` | Enable verbose compilation output |

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+Shift+B` | Compile current file |
| `Ctrl+Space` | Trigger autocomplete |
| `F12` | Go to definition |
| `Ctrl+Hover` | Show hover documentation |

### Commands

Access via Command Palette (`Ctrl+Shift+P`):

- `Hint: Compile Current File` - Compile the current .ht file
- `Hint: Show Tokens` - Display lexical analysis output
- `Hint: Show AST` - Display parsed abstract syntax tree

## Architecture

### Compiler Phases

The Hint compiler follows a traditional three-phase compilation process:

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  Source     │────▶│  Lexer      │────▶│  Parser     │────▶│  Code Gen   │
│  (.ht)      │     │  (Tokens)   │     │  (AST)      │     │  (Assembly) │
└─────────────┘     └─────────────┘     └─────────────┘     └─────────────┘
                                                                   │
                                                                   ▼
┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  Executable │◀────│  Linked     │◀────│  Assembled  │◀────│  NASM       │
│  (.exe)     │     │  Binary     │     │  Object     │     │  Assembly   │
└─────────────┘     └─────────────┘     └─────────────┘     └─────────────┘
```

#### Phase 1: Lexical Analysis

The lexer (`lexer.rs`) converts source text into tokens:

```rust
// Input
Say "Hello, world!".

// Output tokens
Word("say")
String("\"Hello, world!\"")
Period
EOF
```

#### Phase 2: Parsing

The parser (`parser.rs`) converts tokens into an Abstract Syntax Tree (AST):

```rust
// Input tokens
Word("say"), String("\"Hello\""), Period

// Output AST
Speak("Hello")
```

#### Phase 3: Code Generation

The code generator (`codegen.rs`) converts the AST to x86_64 assembly:

```nasm
; Say "Hello"
sub rsp, 40
mov rcx, -11
call GetStdHandle
mov rcx, rax
lea rdx, [str_0]
mov r8, str_0_len
xor r9, r9
call WriteFile
add rsp, 40
```

### Assembly Output

The generated assembly follows Windows x64 calling convention:

- **RCX, RDX, R8, R9**: First four integer/pointer arguments
- **32-byte shadow space**: Reserved on stack before calls
- **16-byte stack alignment**: Required before calls

#### Memory Layout

```
section .data          ; String literals
    str_0: db "Hello, world!", 0
    str_0_len: equ $ - str_0 - 1

section .bss           ; Variables
    var_answer: resq 1

section .text          ; Code
    global main
main:
    ; ... program code ...
```

## Examples

### Hello World

```hint
Say "Hello, World!".
Stop the program.
```

**Compile and run**:
```bash
hintc hello.ht
./hello.exe
# Output: Hello, World!
```

### Variables

```hint
Keep the number 42 in mind as the answer.
Keep the number -17 in mind as the temperature.
Say "The answer is stored.".
Say "The temperature is recorded.".
Stop the program.
```

### Complete Program

```hint
Say "Welcome to Hint Programming!".
Keep the number 10 in mind as the counter.
Keep the number 100 in mind as the limit.
Say "Counter initialized to 10.".
Say "Limit set to 100.".
Say "Program completed successfully.".
Stop the program.
```

### Fibonacci Example

```hint
Say "Computing Fibonacci sequence...".
Keep the number 0 in mind as the first.
Keep the number 1 in mind as the second.
Keep the number 10 in mind as the count.
Say "First 10 Fibonacci numbers:".
Say "0".
Say "1".
Stop the program.
```

## Error Handling

### Lexical Errors

```
Error: Lexical error at position 5: Unterminated string literal
```

**Common causes**:
- Missing closing quote
- Invalid characters
- Newline in string

### Parse Errors

```
Error: Parse error at position 12: Expected '.', found 'Word(Hello)'
```

**Common causes**:
- Missing period at end of statement
- Wrong statement structure
- Unknown keywords

### Compilation Errors

```
Error: NASM not found. Please install NASM and ensure it is in your PATH.
```

**Solution**: Install NASM from [nasm.us](https://www.nasm.us/)

```
Error: MSVC linker (link.exe) not found.
```

**Solution**: Install Visual Studio Build Tools and use Developer Command Prompt

## Limitations

### Current Version (0.1.0)

- **No control flow**: No if/else, loops, or jumps
- **No user input**: Cannot read from stdin
- **No floating-point**: Only integer arithmetic
- **No functions**: No user-defined procedures
- **Windows only**: x86_64 Windows executables
- **Limited types**: No arrays, structs, or complex types

### Planned Features

See [Future Enhancements](#future-enhancements)

## Future Enhancements

### Short-term (v0.2.0)

- [ ] Comments support (`//` and `/* */`)
- [ ] Enhanced error messages with line/column
- [ ] Warning for unused variables
- [ ] Cross-platform support (Linux, macOS)

### Medium-term (v0.3.0)

- [ ] Control flow statements
  ```hint
  If the number is greater than 0, say "Positive".
  Repeat 10 times: Say "Hello".
  ```
- [ ] User input
  ```hint
  Ask for a number and keep it in mind as the input.
  ```
- [ ] Arithmetic expressions
  ```hint
  Keep the sum of 5 and 3 in mind as the result.
  ```

### Long-term (v1.0.0)

- [ ] Functions/procedures
- [ ] Arrays and lists
- [ ] String manipulation
- [ ] File I/O
- [ ] Standard library

## Troubleshooting

### Common Issues

#### "NASM not found"

**Problem**: NASM assembler is not installed or not in PATH

**Solution**:
1. Download NASM from [nasm.us](https://www.nasm.us/)
2. Install to default location
3. Add to PATH: `setx PATH "%PATH%;C:\Program Files\NASM"`
4. Restart terminal/VS Code

#### "link.exe not found"

**Problem**: MSVC linker is not available

**Solution**:
1. Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/)
2. Use "Developer Command Prompt for VS"
3. Or add to PATH: `C:\Program Files (x86)\Microsoft Visual Studio\2019\BuildTools\VC\Tools\MSVC\<version>\bin\Hostx64\x64`

#### Parse errors with variables

**Problem**: Variable name format incorrect

**Solution**: Ensure variable names:
- Start with a letter
- Contain only letters, numbers, and underscores
- Are preceded by "the" in the statement

```hint
// Wrong
Keep the number 42 in mind as my_var.

// Correct
Keep the number 42 in mind as the my_var.
```

### Debugging Tips

1. **Use --tokens**: See how your code is tokenized
   ```bash
   hintc --tokens program.ht
   ```

2. **Use --ast**: View the parsed structure
   ```bash
   hintc --ast program.ht
   ```

3. **Use --keep**: Preserve intermediate files
   ```bash
   hintc --keep program.ht
   # Examines program.asm, program.obj
   ```

4. **Use --verbose**: See compilation progress
   ```bash
   hintc --verbose program.ht
   ```

5. **Check VS Code Output**: View LSP logs
   - View → Output → Hint Language Server

## Contributing

### Project Structure

```
Hint/
├── hintc/                    # Rust compiler
│   ├── src/
│   │   ├── main.rs          # CLI entry point
│   │   ├── lexer.rs         # Lexical analysis
│   │   ├── parser.rs        # Parsing
│   │   ├── codegen.rs       # Code generation
│   │   └── lsp.rs           # Language server
│   ├── Cargo.toml
│   └── documentation.md
├── hint-vscode/              # VS Code extension
│   ├── src/
│   │   └── extension.ts     # Extension entry
│   ├── syntaxes/
│   │   └── hint.json        # Syntax highlighting
│   ├── assets/
│   │   └── logo.svg         # Extension logo
│   └── package.json
└── plans/                    # Future plans
```

### Building Everything

```bash
# Build compiler
cd hintc
cargo build --release

# Build extension
cd ../hint-vscode
npm install
npm run compile

# Package extension (optional)
vsce package
```

### Running Tests

```bash
# Compiler tests
cd hintc
cargo test

# Extension tests
cd ../hint-vscode
npm test
```

### Code Style

- **Rust**: Follow Rustfmt defaults
- **TypeScript**: ESLint rules in extension
- **Documentation**: Markdown with clear examples

### Submitting Changes

1. Fork the repository
2. Create a feature branch
3. Make changes with tests
4. Submit a pull request

## License

MIT License - See LICENSE file for details.

## Acknowledgments

- NASM Team - [The Netwide Assembler](https://www.nasm.us/)
- Rust Team - [Rust Programming Language](https://www.rust-lang.org/)
- Microsoft - [VS Code](https://code.visualstudio.com/)

---

**Happy Hinting!** 🚀
