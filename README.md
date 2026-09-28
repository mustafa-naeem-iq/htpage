# htpage

A high-performance, unified markup language and multi-target compiler suite.

## Architecture

This monorepo provides:
- **`crates/htpage_core`**: Pure Rust compiler engine (Lexer, Parser, AST, HTML/CSS code generator). Zero external runtime dependencies.
- **`crates/htpage_cli`**: Standalone terminal executable for compiling `.htpage` files to `.html`.
- **`crates/htpage_wasm`**: WebAssembly bindings via `wasm-bindgen` (~80KB standalone `.wasm`).
- **`packages/vite-plugin-htpage`**: Vite plugin with in-process Wasm compilation and instant HMR.
- **`crates/htpage_rb`**: Native C-extension dynamic library via `rb-sys`.
- **`gems/htpage`**: Ruby Gem providing native in-process Rails `ActionView` template handler (`.html.htpage`).

---

## 1. Standalone CLI Usage

```bash
cargo run -p htpage_cli -- <input.htpage> [output.html]
```

---

## 2. Vite Integration (`vite-plugin-htpage`)

Install and register in your `vite.config.js`:

```javascript
import { defineConfig } from 'vite';
import { htpage } from 'vite-plugin-htpage';

export default defineConfig({
  plugins: [htpage()]
});
```

Import `.htpage` files directly in your frontend code:

```javascript
import html from './components/card.htpage';

document.getElementById('app').innerHTML = html;
```

---

## 3. Ruby on Rails Integration (`gem 'htpage'`)

Add to your `Gemfile`:

```ruby
gem 'htpage', path: 'path/to/htpage/gems/htpage'
```

Create views named with the `.html.htpage` extension:

```htpage
# app/views/home/index.html.htpage
doctype: html
html {
    head {
        title: Welcome
    }
    body {
        h1 { Hello from Rails, color: royalblue }
        p { Rendered in-process at native C speed }
    }
}
```

Rails automatically recognizes `.htpage` templates and renders them in-process with zero subprocess overhead.
