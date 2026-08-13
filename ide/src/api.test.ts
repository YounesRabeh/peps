import { beforeEach, describe, expect, it, vi } from "vitest";

const wasmMocks = vi.hoisted(() => ({
  initialize: vi.fn(),
  run: vi.fn()
}));

vi.mock("./wasm/peps.js", () => ({
  default: wasmMocks.initialize,
  run_peps: wasmMocks.run
}));

import { runPepsSource } from "./api";

describe("runPepsSource", () => {
  beforeEach(() => {
    wasmMocks.initialize.mockResolvedValue(undefined);
    wasmMocks.run.mockReset();
  });

  it("runs source through WebAssembly and parses its response", async () => {
    wasmMocks.run.mockReturnValue(
      JSON.stringify({ ok: true, output: ["7"], diagnostics: [] })
    );

    await expect(runPepsSource("📢 7️⃣")).resolves.toEqual({
      ok: true,
      output: ["7"],
      diagnostics: []
    });
    expect(wasmMocks.run).toHaveBeenCalledWith("📢 7️⃣", "[]");
  });

  it("passes terminal input lines to WebAssembly", async () => {
    wasmMocks.run.mockReturnValue(
      JSON.stringify({ ok: true, output: ["42"], diagnostics: [], inputRequest: null })
    );

    await runPepsSource("🐶 🟰 ⌨️ 🔢", ["42"]);
    expect(wasmMocks.run).toHaveBeenCalledWith("🐶 🟰 ⌨️ 🔢", '["42"]');
  });
});
