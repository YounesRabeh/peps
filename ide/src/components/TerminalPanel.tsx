import { useState, type FormEvent } from "react";
import type { RunResponse } from "../api";

type TerminalPanelProps = {
  running: boolean;
  response: RunResponse | null;
  error: string | null;
  transcript: TerminalEntry[];
  onSubmitInput: (value: string) => void | Promise<void>;
  onClear: () => void;
};

export type TerminalEntry = {
  kind: "input" | "output";
  value: string;
};

export function TerminalPanel({
  running,
  response,
  error,
  transcript,
  onSubmitInput,
  onClear
}: TerminalPanelProps) {
  const [value, setValue] = useState("");
  const hasDiagnostics = Boolean(response && response.diagnostics.length > 0);
  const waitingForInput = response?.inputRequest ?? null;
  const isSuccess = Boolean(!running && !error && response?.ok && !hasDiagnostics);

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!waitingForInput || running) return;
    onSubmitInput(value);
    setValue("");
  }

  return (
    <aside className="terminal-panel" id="terminal-panel">
      <div className="terminal-header">
        <h2>Terminal</h2>
        <div className="terminal-actions">
          <button className="terminal-clear" onClick={onClear} type="button">
            Clear
          </button>
          <span
            className={`status-pill ${
              running
                ? "status-running"
                : waitingForInput
                  ? "status-waiting"
                  : isSuccess
                    ? "status-ok"
                    : hasDiagnostics || error
                      ? "status-error"
                      : "status-idle"
            }`}
          >
            {running
              ? "Running"
              : waitingForInput
                ? "Input"
                : isSuccess
                  ? "Success"
                  : hasDiagnostics || error
                    ? "Issues"
                    : "Idle"}
          </span>
        </div>
      </div>

      <div className="terminal-screen" aria-live="polite" role="log">
        {transcript.map((entry, index) => (
          <div
            className={entry.kind === "input" ? "terminal-input-line" : "terminal-output-line"}
            key={`${entry.kind}-${entry.value}-${index}`}
          >
            {entry.kind === "input" && <span aria-hidden="true">❯ </span>}
            {entry.value}
          </div>
        ))}
        {running && <div className="terminal-muted">Running...</div>}
        {!running && error && <div className="terminal-error">IDE error: {error}</div>}
        {!running && !error && !response && (
          <div className="terminal-muted">Press Run to start the program.</div>
        )}
        {!running && response?.output.map((line, index) => (
          <div className="terminal-output-line" key={`${line}-${index}`}>
            {line}
          </div>
        ))}
        {!running &&
          response?.ok &&
          response.output.length === 0 &&
          !transcript.some((entry) => entry.kind === "output") && (
          <div className="terminal-muted">Program finished with no output.</div>
        )}
        {!running && waitingForInput && (
          <div className="terminal-prompt">Waiting for {waitingForInput} input…</div>
        )}
        {!running && response && response.diagnostics.map((diagnostic, index) => (
          <div className="terminal-error" key={`${diagnostic.message}-${index}`}>
            {formatDiagnostic(diagnostic)}
          </div>
        ))}
      </div>

      <form className="terminal-input" onSubmit={handleSubmit}>
        <label htmlFor="terminal-value">
          {waitingForInput ? `${waitingForInput} input` : "Program input"}
        </label>
        <div className="terminal-input-row">
          <span aria-hidden="true">❯</span>
          <input
            aria-label="Terminal input"
            autoComplete="off"
            disabled={!waitingForInput || running}
            id="terminal-value"
            onChange={(event) => setValue(event.currentTarget.value)}
            placeholder={waitingForInput ? `Enter ${waitingForInput}` : "Run a program that uses ⌨️"}
            value={value}
          />
          <button disabled={!waitingForInput || running} type="submit">
            Send
          </button>
        </div>
      </form>
    </aside>
  );
}

function formatDiagnostic(diagnostic: RunResponse["diagnostics"][number]): string {
  if (diagnostic.kind === "runtime") {
    const location = diagnostic.line ? ` at line ${diagnostic.line}` : "";
    return `runtime error${location}: ${diagnostic.message}`;
  }

  return `${formatLocation(diagnostic.line, diagnostic.column)}: ${diagnostic.message}`;
}

function formatLocation(line?: number | null, column?: number | null): string {
  if (line && column) return `line ${line}, column ${column}`;
  if (line) return `line ${line}`;
  return "compile error";
}
