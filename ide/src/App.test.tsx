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

  it("loads the complete overview by default", () => {
    render(<App />);

    const source = (screen.getByLabelText("mock editor") as HTMLTextAreaElement).value;
    expect(source).toContain("Peps overview");
    expect(source).toContain("🧩");
    expect(source).toContain("⌨️ 🔤");
    expect(source).toContain("🔄 🔢");
    expect(source).toContain("🗺️");
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

  it("collapses and restores the terminal and documentation panels", () => {
    render(<App />);

    fireEvent.click(screen.getByRole("button", { name: "Hide panels" }));
    expect(screen.queryByLabelText("Run results and documentation")).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Show panels" }));
    expect(screen.getByLabelText("Run results and documentation")).toBeInTheDocument();
  });

  it("shows the terminal when Run is pressed after panels are hidden", async () => {
    runPepsSourceMock.mockResolvedValueOnce({ ok: true, output: [], diagnostics: [] });

    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "Hide panels" }));
    fireEvent.click(screen.getByRole("button", { name: "Run ▶" }));

    expect(screen.getByLabelText("Run results and documentation")).toBeInTheDocument();
    expect(screen.getByRole("separator", { name: "Resize terminal and documentation" })).toBeInTheDocument();

    await waitFor(() => {
      expect(screen.getByText("Program finished with no output.")).toBeInTheDocument();
    });
  });

  it("submits requested terminal input and resumes execution", async () => {
    runPepsSourceMock
      .mockResolvedValueOnce({
        ok: false,
        output: ["Enter a number"],
        diagnostics: [],
        inputRequest: "integer"
      })
      .mockResolvedValueOnce({
        ok: true,
        output: ["Enter a number", "answer: 42"],
        diagnostics: [],
        inputRequest: null
      });

    render(<App />);
    fireEvent.click(screen.getByRole("button", { name: "Run ▶" }));

    await screen.findByText("Waiting for integer input…");
    fireEvent.change(screen.getByLabelText("Terminal input"), {
      target: { value: "42" }
    });
    fireEvent.click(screen.getByRole("button", { name: "Send" }));

    await waitFor(() => {
      expect(runPepsSourceMock).toHaveBeenLastCalledWith(expect.any(String), ["42"]);
      expect(screen.getByText("answer: 42")).toBeInTheDocument();
    });

    const terminal = screen.getByRole("log");
    expect(terminal.textContent?.indexOf("Enter a number")).toBeLessThan(
      terminal.textContent?.indexOf("❯ 42") ?? -1
    );
    expect(terminal.textContent?.indexOf("❯ 42")).toBeLessThan(
      terminal.textContent?.indexOf("answer: 42") ?? -1
    );
  });
});
