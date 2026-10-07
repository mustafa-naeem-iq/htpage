# VS Code Extension for htpage

Official Visual Studio Code extension providing first-class language support for the **htpage** unified markup language.

## Features
- **Syntax Highlighting**: Pure TextMate grammar covering element tags, property assignments, grouping parentheses, pseudo-classes, and comments.
- **Embedded Languages**: Native CSS highlighting inside `css { ... }` blocks and JavaScript highlighting inside `javascript { ... }` blocks.
- **Diagnostics**: Real-time syntax and semantic validation powered by `htpage_lsp`.
- **Auto-Completion**: Context-aware suggestions for HTML tags, global/tag-specific attributes, boolean flags, pseudo-classes, and CSS properties.
- **Hover Information**: Documentation and element breakdown on hover.
- **Outline & Breadcrumbs**: Hierarchical document symbols representing the element tree.
- **Bracket Matching & Indentation**: Auto-closing braces, parenthesis pairs, and smart block indentation.

## Requirements
The extension connects to the `htpage_lsp` binary. By default, it searches:
1. Workspace `target/release/htpage_lsp` or `target/debug/htpage_lsp`
2. System `PATH` (`htpage_lsp`)
3. User setting `htpage.lsp.serverPath`
