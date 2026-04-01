# Hint Compiler

**Zero-dependency compiler for the Hint programming language.**

[![Version](https://img.shields.io/badge/version-0.2.0-blue.svg)](https://github.com/hint-lang/hintc/releases)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](../LICENSE)
[![Build Status](https://img.shields.io/github/actions/workflow/status/hint-lang/hintc/ci.yml)](https://github.com/hint-lang/hintc/actions)

---

## 🚀 Overview

The Hint compiler (`hintc`) compiles Hint source code to:
- **WebAssembly** (`.wasm`) - For web browsers
- **Native executables** (`.exe`, ELF, Mach-O) - For Windows, macOS, Linux

**Written in Rust** using the Cranelift code generator.

---

## 📦 Installation

### From Source

```bash
git clone https://github.com/hint-lang/hintc.git
cd hintc/hintc
cargo build --release
```

The binary will be at `target/release/hintc` (or `hintc.exe` on Windows).

### From npm (Coming Soon)

```bash
npm install -g hintc
```

---

## 🛠️ Usage

### Compile to WASM

```bash
hintc --target wasm32 input.ht -o output.wasm
```

### Compile to Native

```bash
# Windows
hintc --target native input.ht -o output.exe

# Linux/macOS
hintc --target native input.ht -o output
```

### Run REPL

```bash
hintc --repl
```

---

## 📁 Project Structure

```
hintc/
├── src/
│   ├── lexer.rs              # Tokenization
│   ├── parser.rs             # AST generation
│   ├── semantics/            # Type checking
│   │   ├── mod.rs
│   │   ├── types.rs
│   │   ├── symbols.rs
│   │   ├── checker.rs
│   │   └── error.rs
│   ├── ir/                   # Intermediate representation
│   │   └── mod.rs
│   ├── codegen/              # Code generation
│   │   ├── mod.rs
│   │   ├── wasm/             # WASM backend
│   │   └── native/           # Native backend (Cranelift)
│   ├── stdlib/               # Standard library
│   │   ├── mod.rs
│   │   ├── core.rs
│   │   ├── io.rs
│   │   ├── net.rs
│   │   └── wasm.rs
│   ├── diagnostics/          # Error messages
│   │   ├── mod.rs
│   │   ├── diagnostic.rs
│   │   ├── engine.rs
│   │   ├── codes.rs
│   │   ├── suggestions.rs
│   │   └── render.rs
│   ├── lsp.rs                # Language server
│   ├── compiler.rs           # Main compiler
│   ├── target.rs             # Target abstraction
│   ├── main.rs               # CLI entry
│   └── lib.rs                # Library entry
├── tests/                    # Test files
├── Cargo.toml                # Dependencies
└── README.md                 # This file
```

---

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test lexer

# Run with output
cargo test -- --nocapture
```

### Test Files

Test `.ht` files are in the `tests/` folder:
- `fibonacci.ht` - Fibonacci sequence
- `hello_world.ht` - Hello World example

---

## 🏗️ Architecture

See [ARCHITECTURE.md](ARCHITECTURE.md) for detailed compiler architecture.

### Compilation Pipeline

```
┌─────────────┐
│ Source (.ht)│
└──────┬──────┘
       │
       ▼
┌─────────────┐
│    Lexer    │ → Tokens
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   Parser    │ → AST
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  Semantics  │ → Typed AST
└──────┬──────┘
       │
       ▼
┌─────────────┐
│     IR      │ → HIR
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  Codegen    │ → WASM / Native
└─────────────┘
```

---

## 📚 Documentation

| Document | Description |
|----------|-------------|
| [README.md](../README.md) | Main project README |
| [ARCHITECTURE.md](ARCHITECTURE.md) | Compiler architecture |
| [documentation.md](documentation.md) | Compiler documentation |

---

## 🤝 Contributing

1. Fork the repo
2. Create a branch (`git checkout -b feature/my-feature`)
3. Make your changes
4. Run tests (`cargo test`)
5. Submit a PR

---

## 📄 License

MIT License - See [LICENSE](../LICENSE) for details.

---

**© 2026 Hint Language Team**
