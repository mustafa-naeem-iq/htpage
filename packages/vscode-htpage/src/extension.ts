import * as path from 'path';
import * as fs from 'fs';
import * as os from 'os';
import { workspace, ExtensionContext } from 'vscode';
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from 'vscode-languageclient/node';

let client: LanguageClient | undefined;

export function activate(context: ExtensionContext) {
  const config = workspace.getConfiguration('htpage');
  const customPath = config.get<string>('lsp.serverPath');

  let serverCommand = customPath;
  if (!serverCommand || serverCommand.trim() === '') {
    const candidatePaths = [
      path.join(os.homedir(), '.cargo/bin/htpage_lsp'),
      path.join(context.extensionPath, '../../target/release/htpage_lsp'),
      path.join(context.extensionPath, '../../target/debug/htpage_lsp'),
      'htpage_lsp',
    ];

    for (const candidate of candidatePaths) {
      if (candidate === 'htpage_lsp' || fs.existsSync(candidate)) {
        serverCommand = candidate;
        break;
      }
    }
  }

  const serverOptions: ServerOptions = {
    command: serverCommand || 'htpage_lsp',
    args: [],
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: 'file', language: 'htpage' }],
    synchronize: {
      fileEvents: workspace.createFileSystemWatcher('**/*.htpage'),
    },
  };

  client = new LanguageClient(
    'htpageLsp',
    'htpage Language Server',
    serverOptions,
    clientOptions
  );

  client.start();
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
