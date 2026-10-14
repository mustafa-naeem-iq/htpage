# htpage

A high-performance, unified markup language and multi-target compiler suite designed to eliminate HTML and CSS boilerplate through radical brevity and clean syntax.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

---

## Quick Start: 4 Ways to Use htpage

### 1. Command Line & Language Server (Homebrew)

Install both the standalone compiler CLI (`htpage`) and the Language Server Protocol (`htpage_lsp`) in one step:

```bash
brew tap mustafa-naeem-iq/htpage
brew install htpage
```

Compile any `.htpage` document to standards-compliant HTML:

```bash
htpage index.htpage index.html
```

---

### 2. Visual Studio Code Extension

First-class language support for VS Code including syntax highlighting, embedded CSS/JS, real-time diagnostics, and context-aware auto-completion:

- Search for **`htpage`** in the Visual Studio Code Extensions view (`Ctrl+Shift+X` / `Cmd+Shift+X`).
- Or install via CLI:
  ```bash
  code --install-extension htpage.vscode-htpage
  ```

*The extension automatically discovers `htpage_lsp` installed via Homebrew at `/opt/homebrew/bin/htpage_lsp`.*

---

### 3. Vite Frontend Plugin (npm)

Zero-runtime, in-process compilation of `.htpage` templates in Vite projects using an embedded WebAssembly core:

```bash
npm install -D vite-plugin-htpage
```

Configure `vite.config.ts`:

```typescript
import { defineConfig } from 'vite';
import { htpage } from 'vite-plugin-htpage';

export default defineConfig({
  plugins: [htpage()]
});
```

Import `.htpage` files directly into your JavaScript or TypeScript modules:

```typescript
import cardHtml from './components/card.htpage';

document.querySelector('#app')!.innerHTML = cardHtml;
```

---

### 4. Ruby on Rails Template Handler (RubyGems)

Blazing fast, zero-IPC native C-extension (`rb-sys`) registering `.html.htpage` templates in Rails `ActionView`:

Add to your `Gemfile`:

```ruby
gem "htpage"
```

Create templates in `app/views/` using the `.html.htpage` extension:

```htpage
# app/views/pages/home.html.htpage
doctype: html
html {
  head {
    title: Home
  }
  body {
    h1 { Welcome to Rails with htpage, color: royalblue }
    p { Rendered in-process at native C speed with zero subprocess overhead. }
  }
}
```

---

## Syntax Overview

`htpage` collapses HTML structure and CSS styling into a single syntax. CSS properties are treated as direct element attributes, and primary text values sit cleanly at the top of the block:

```htpage
div {
  h1 { Hello World, color: #2563eb, font-size: 2rem }
  p {
    A unified markup language that replaces verbose HTML tags.
    color: #4b5563, line-height: 1.6
  }
  button {
    Click Me,
    background: #2563eb, color: white, border-radius: 8px, padding: 8px 16px,
    hover { background: #1d4ed8 }
  }
}
```

---

## Monorepo Architecture

- **`crates/htpage_core`**: Pure Rust compiler core (Lexer, tolerant Parser, AST, Validator, HTML/CSS Bundler).
- **`crates/htpage_cli`**: Standalone terminal compiler (`htpage`).
- **`crates/htpage_lsp`**: Native Language Server Protocol implementation (`htpage_lsp`).
- **`crates/htpage_wasm`**: WebAssembly compilation target.
- **`crates/htpage_rb`**: Native C-extension dynamic library via `rb-sys`.
- **`packages/vscode-htpage`**: VS Code language client & TextMate grammar.
- **`packages/vite-plugin-htpage`**: Vite plugin with vendored Wasm.
- **`gems/htpage`**: Packaged Ruby gem with Rails ActionView template handler.

---

## License

MIT © [Mustafa Naeem](https://github.com/mustafa-naeem-iq)
