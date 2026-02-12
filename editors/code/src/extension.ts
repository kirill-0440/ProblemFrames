import * as fs from "fs";
import { workspace, ExtensionContext } from "vscode";
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  Executable,
  TransportKind,
} from "vscode-languageclient/node";

let client: LanguageClient;

export function activate(context: ExtensionContext) {
  // Prefer bundled binary from the extension package, fallback to PATH.
  const config = workspace.getConfiguration("problemFrames");
  const bundledServerPath = context.asAbsolutePath("pf_lsp");
  const defaultServerPath = fs.existsSync(bundledServerPath)
    ? bundledServerPath
    : "pf_lsp";
  const serverPath = config.get<string>("serverPath") || defaultServerPath;

  const run: Executable = {
    command: serverPath,
    transport: TransportKind.stdio,
  };

  const serverOptions: ServerOptions = {
    run,
    debug: run,
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "pf" }],
  };

  client = new LanguageClient(
    "problemFrames",
    "Problem Frames LSP",
    serverOptions,
    clientOptions,
  );

  client.start();
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
