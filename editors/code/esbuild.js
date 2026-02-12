const esbuild = require("esbuild");
const fs = require("fs");

const isWatch = process.argv.includes("--watch");
const isProduction = process.argv.includes("--production");

const buildOptions = {
  entryPoints: ["src/extension.ts"],
  bundle: true,
  platform: "node",
  format: "cjs",
  target: "node16",
  outfile: "out/extension.js",
  sourcemap: !isProduction,
  external: ["vscode"],
  logLevel: "info",
};

async function main() {
  if (isProduction) {
    fs.rmSync("out/extension.js.map", { force: true });
  }

  if (isWatch) {
    const context = await esbuild.context(buildOptions);
    await context.watch();
    return;
  }

  await esbuild.build(buildOptions);
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
