import initializePeps, { run_peps as runPeps } from "./wasm/peps.js";

export type IdeDiagnostic = {
  message: string;
  line?: number | null;
  column?: number | null;
  start?: number | null;
  end?: number | null;
};

export type RunResponse = {
  ok: boolean;
  output: string[];
  diagnostics: IdeDiagnostic[];
  inputRequest?: "text" | "integer" | "float" | "boolean" | null;
};

let wasmInitialization: Promise<unknown> | null = null;

function initializeWasm() {
  wasmInitialization ??= initializePeps();
  return wasmInitialization;
}

export async function runPepsSource(
  source: string,
  inputs: string[] = []
): Promise<RunResponse> {
  await initializeWasm();
  return JSON.parse(runPeps(source, JSON.stringify(inputs))) as RunResponse;
}
