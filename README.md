# htpage

A high-performance, unified markup language and multi-target compiler suite.

## Architecture

This Cargo workspace provides:
- **`crates/htpage_core`**: The pure Rust compiler engine (Lexer, Parser, AST, HTML/CSS code generator). Zero external runtime dependencies.
- **`crates/htpage_cli`**: Standalone terminal executable for compiling `.htpage` files to `.html`.
- **`crates/htpage_wasm`** *(planned)*: WebAssembly bindings for Vite and frontend toolchains.
- **`crates/htpage_rb`** *(planned)*: Ruby C-extension native gem for Rails `ActionView`.

## Usage

```bash
cargo run -p htpage_cli -- <input.htpage> [output.html]
```
