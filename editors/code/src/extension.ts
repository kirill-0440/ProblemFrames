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
  const bundledCandidates =
    process.platform === "win32"
      ? ["pf_lsp.exe", "pf_lsp"]
      : ["pf_lsp", "pf_lsp.exe"];
  const bundledServerPath = bundledCandidates
    .map((candidate) => context.asAbsolutePath(candidate))
    .find((candidatePath) => fs.existsSync(candidatePath));
  const defaultServerPath = bundledServerPath ?? "pf_lsp";
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
