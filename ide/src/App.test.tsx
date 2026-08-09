import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { App } from "./App";
import { runPepsSource } from "./api";

vi.mock("./api", () => ({
  runPepsSource: vi.fn()
}));

const runPepsSourceMock = vi.mocked(runPepsSource);

vi.mock("@monaco-editor/react", () => ({
  default: ({ value, onChange }: { value: string; onChange: (value: string) => void }) => (
    <textarea
      aria-label="mock editor"
      value={value}
      onChange={(event) => onChange(event.currentTarget.value)}
    />
  )
}));

describe("App", () => {
  beforeEach(() => {
    runPepsSourceMock.mockReset();
  });

  it("runs source and renders output", async () => {
    runPepsSourceMock.mockResolvedValueOnce({
      ok: true,
      output: ["5"],
      diagnostics: []
    });

    render(<App />);

    const button = screen.getByRole("button", { name: "Run ▶" });
    fireEvent.click(button);

    expect(button).toBeDisabled();
    expect(screen.getAllByText("Running...").length).toBeGreaterThan(0);

    await waitFor(() => {
      expect(screen.getByText("5")).toBeInTheDocument();
    });
  });

  it("keeps the numbered documentation panel in the IDE and loads examples", () => {
    render(<App />);

    expect(screen.getByRole("heading", { name: "1. Variables" })).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "7 Functions" }));
    expect(screen.getByRole("heading", { name: "7. Functions" })).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Load example into editor" }));
    expect((screen.getByLabelText("mock editor") as HTMLTextAreaElement).value).toContain("🧩");
  });

  it("collapses and restores the output and documentation panels", () => {
    render(<App />);

    fireEvent.click(screen.getByRole("button", { name: "Hide panels" }));
    expect(screen.queryByLabelText("Run results and documentation")).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Show panels" }));
    expect(screen.getByLabelText("Run results and documentation")).toBeInTheDocument();
  });

  it("shows the output panel when Run is pressed after panels are hidden", async () => {
    runPepsSourceMock.mockResolvedValueOnce({ ok: true, output: [], diagnostics: [] });

    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "Hide panels" }));
    fireEvent.click(screen.getByRole("button", { name: "Run ▶" }));

    expect(screen.getByLabelText("Run results and documentation")).toBeInTheDocument();
    expect(screen.getByRole("separator", { name: "Resize output and documentation" })).toBeInTheDocument();

    await waitFor(() => {
      expect(screen.getByText("Program finished with no output.")).toBeInTheDocument();
    });
  });
});
