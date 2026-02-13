import * as fs from "fs";
import { workspace, ExtensionContext, commands, window } from "vscode";
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
  context.subscriptions.push(client);

  const impactCommand = commands.registerCommand(
    "problemFrames.showImpactedRequirements",
    async () => {
      const editor = window.activeTextEditor;
      if (!editor || editor.document.languageId !== "pf") {
        void window.showInformationMessage(
          "Open a PF model and place cursor on a domain or requirement symbol.",
        );
        return;
      }

      try {
        const result = await client.sendRequest<{
          seedKind: string;
          seedId: string;
          impactedRequirements: string[];
          maxHops: number;
        } | null>("problemFrames/impactRequirements", {
          textDocument: { uri: editor.document.uri.toString() },
          position: editor.selection.active,
          maxHops: 2,
        });

        if (!result) {
          void window.showInformationMessage(
            "No impact seed resolved at cursor position.",
          );
          return;
        }

        const impacted =
          result.impactedRequirements.length === 0
            ? "(none)"
            : result.impactedRequirements.join(", ");
        void window.showInformationMessage(
          `Impact (${result.seedKind}:${result.seedId}, hops=${result.maxHops}): ${impacted}`,
        );
      } catch (error) {
        const message =
          error instanceof Error
            ? error.message
            : "Unknown impact request error";
        void window.showErrorMessage(
          `Failed to query impacted requirements: ${message}`,
        );
      }
    },
  );
  context.subscriptions.push(impactCommand);
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
