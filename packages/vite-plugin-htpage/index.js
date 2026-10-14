let compile;
try {
  compile = require('./wasm/htpage_wasm.js').compile;
} catch {
  try {
    compile = require('htpage_wasm').compile;
  } catch {
    compile = require('../../crates/htpage_wasm/pkg/htpage_wasm.js').compile;
  }
}

function htpage() {
  return {
    name: 'vite-plugin-htpage',
    enforce: 'pre',
    transform(code, id) {
      if (!id.endsWith('.htpage')) return null;
      try {
        const html = compile(code);
        return {
          code: `export default ${JSON.stringify(html)};`,
          map: null,
        };
      } catch (err) {
        this.error(err);
      }
    },
    handleHotUpdate({ file, server }) {
      if (file.endsWith('.htpage')) {
        server.ws.send({
          type: 'full-reload',
          path: '*'
        });
      }
    }
  };
}

module.exports = { htpage };
module.exports.default = htpage;
