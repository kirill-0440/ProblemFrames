import * as path from 'path';
import { workspace, ExtensionContext } from 'vscode';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    Executable,
    TransportKind
} from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: ExtensionContext) {
    // Get the server path from config or default to 'pf_lsp' in PATH
    const config = workspace.getConfiguration('problemFrames');
    const serverPath = config.get<string>('serverPath') || 'pf_lsp';

    // If we are in dev mode, we might want to look in target/debug
    // But for general use 'pf_lsp' in path is best.

    const run: Executable = {
        command: serverPath,
        transport: TransportKind.stdio
    };

    const serverOptions: ServerOptions = {
        run,
        debug: run
    };

    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: 'file', language: 'pf' }],
    };

    client = new LanguageClient(
        'problemFrames',
        'Problem Frames LSP',
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
