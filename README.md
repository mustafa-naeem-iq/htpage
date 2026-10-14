<p align="center">
  <img src="assets/HTLogo.png" alt="htpage banner" width="700" />
</p>

<h1 align="center">htpage</h1>

<p align="center">
  <strong>A high-performance, unified markup language and multi-target compiler suite.</strong><br>
  Radical brevity, clean syntax, and zero-boilerplate HTML + CSS.
</p>

<p align="center">
  <a href="https://marketplace.visualstudio.com/items?itemName=mustafa-naeem.vscode-htpage"><img src="https://img.shields.io/visual-studio-marketplace/v/mustafa-naeem.vscode-htpage?color=007ACC&label=VS%20Code" alt="VS Code Marketplace"></a>
  <a href="https://rubygems.org/gems/htpage"><img src="https://img.shields.io/gem/v/htpage?color=E9573F&label=RubyGems" alt="RubyGems"></a>
  <a href="https://www.npmjs.com/package/vite-plugin-htpage"><img src="https://img.shields.io/npm/v/vite-plugin-htpage?color=CB3837&label=npm" alt="npm"></a>
  <a href="https://github.com/mustafa-naeem-iq/homebrew-htpage"><img src="https://img.shields.io/badge/Homebrew-Tap-FBB040?logo=homebrew&logoColor=black" alt="Homebrew Tap"></a>
  <a href="https://opensource.org/licenses/MIT"><img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License"></a>
</p>

---

<table align="center" border="0">
  <tr>
    <td align="center" width="25%">
      <a href="#1-command-line--language-server-homebrew-tap">
        <img src="assets/icons/brew.png" height="70" alt="Homebrew"><br>
        <strong>Homebrew</strong>
      </a>
    </td>
    <td align="center" width="25%">
      <a href="#2-visual-studio-code-extension-vs-code-marketplace">
        <img src="assets/icons/vscode.png" height="70" alt="VS Code"><br>
        <strong>VS Code</strong>
      </a>
    </td>
    <td align="center" width="25%">
      <a href="#3-vite-frontend-plugin-npm">
        <img src="assets/icons/vite.png" height="70" alt="Vite"><br>
        <strong>Vite</strong>
      </a>
    </td>
    <td align="center" width="25%">
      <a href="#4-ruby-on-rails-template-handler-rubygems">
        <img src="assets/icons/ruby.png" height="70" alt="Rails Gem"><br>
        <strong>Rails Gem</strong>
      </a>
    </td>
  </tr>
</table>

---

## 4 Official Distribution Channels

### <img src="assets/icons/brew.png" height="24" valign="middle" /> 1. Command Line & Language Server ([Homebrew Tap](https://github.com/mustafa-naeem-iq/homebrew-htpage))

Install both the standalone compiler CLI (`htpage`) and the Language Server Protocol (`htpage_lsp`) in one step:

```bash
brew tap mustafa-naeem-iq/htpage
brew install htpage
```

> **Note**: Homebrew 4.2+ displays a one-time prompt to trust third-party taps. In headless CI/CD pipelines, pre-trust the tap by running `brew trust --tap mustafa-naeem-iq/htpage` before installation.

Compile any `.htpage` document to standards-compliant HTML:

```bash
htpage index.htpage index.html
```

---

### <img src="assets/icons/vscode.png" height="24" valign="middle" /> 2. Visual Studio Code Extension ([VS Code Marketplace](https://marketplace.visualstudio.com/items?itemName=mustafa-naeem.vscode-htpage))

First-class language support for VS Code including syntax highlighting, embedded CSS/JS, real-time diagnostics, and context-aware auto-completion:

- Install via the [Visual Studio Marketplace](https://marketplace.visualstudio.com/items?itemName=mustafa-naeem.vscode-htpage) by searching **`htpage`** in the Extensions view (`Ctrl+Shift+X` / `Cmd+Shift+X`).
- Or install via CLI:
  ```bash
  code --install-extension mustafa-naeem.vscode-htpage
  ```

*The extension automatically discovers `htpage_lsp` installed via Homebrew at `/opt/homebrew/bin/htpage_lsp`.*

---

### <img src="assets/icons/vite.png" height="24" valign="middle" /> 3. Vite Frontend Plugin ([npm](https://www.npmjs.com/package/vite-plugin-htpage))

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

### <img src="assets/icons/ruby.png" height="24" valign="middle" /> 4. Ruby on Rails Template Handler ([RubyGems](https://rubygems.org/gems/htpage))

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
