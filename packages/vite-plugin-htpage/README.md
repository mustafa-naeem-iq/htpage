# vite-plugin-htpage

Official Vite plugin providing zero-runtime, in-process compilation of `.htpage` files into standard HTML using an embedded WebAssembly core.

## Installation

```bash
npm install -D vite-plugin-htpage
# or
pnpm add -D vite-plugin-htpage
# or
yarn add -D vite-plugin-htpage
```

## Usage

In your `vite.config.js` or `vite.config.ts`:

```typescript
import { defineConfig } from 'vite';
import { htpage } from 'vite-plugin-htpage';

export default defineConfig({
  plugins: [
    htpage()
  ]
});
```

Now you can import `.htpage` files directly in your application:

```typescript
import html from './views/index.htpage';

document.querySelector('#app')!.innerHTML = html;
```

## Features

- **Embedded WebAssembly Engine**: Runs the pure Rust compiler core in-process via WebAssembly. Zero external binary dependencies.
- **Fast Full Reload HMR**: Watches `.htpage` file changes and triggers instant Vite HMR reloading.
- **TypeScript Support**: Includes built-in `.d.ts` declarations.

## License

MIT © [Mustafa Naeem](https://github.com/mustafa-naeem-iq)
