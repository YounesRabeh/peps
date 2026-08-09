import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";

const ideDirectory = fileURLToPath(new URL("..", import.meta.url));
const workspaceDirectory = fileURLToPath(new URL("../..", import.meta.url));
const executableSuffix = process.platform === "win32" ? ".cmd" : "";
const viteArguments = process.argv.slice(2);

if (viteArguments[0] === "--") viteArguments.shift();

function start(command, arguments_, options) {
  return spawn(`${command}${executableSuffix}`, arguments_, {
    stdio: "inherit",
    ...options,
  });
}

const compiler = start("cargo", ["run", "--bin", "peps-ide"], {
  cwd: workspaceDirectory,
  env: { ...process.env, PEPS_IDE_NO_BROWSER: "1" },
});
const frontend = start("pnpm", ["exec", "vite", ...viteArguments], {
  cwd: ideDirectory,
});

let stopping = false;

function stop(exitCode) {
  if (stopping) return;
  stopping = true;
  compiler.kill();
  frontend.kill();
  process.exit(exitCode);
}

compiler.on("error", (error) => {
  console.error(`Could not start the Peps compiler server: ${error.message}`);
  stop(1);
});

frontend.on("error", (error) => {
  console.error(`Could not start the Vite development server: ${error.message}`);
  stop(1);
});

compiler.on("exit", (code, signal) => {
  if (!stopping) {
    console.error(
      `Peps compiler server stopped${signal ? ` (${signal})` : ""}${
        code === null ? "" : ` with exit code ${code}`
      }.`,
    );
    stop(code ?? 1);
  }
});

frontend.on("exit", (code, signal) => {
  if (!stopping) {
    console.error(
      `Vite development server stopped${signal ? ` (${signal})` : ""}${
        code === null ? "" : ` with exit code ${code}`
      }.`,
    );
    stop(code ?? 1);
  }
});

process.on("SIGINT", () => stop(0));
process.on("SIGTERM", () => stop(0));
