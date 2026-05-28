import type { RunResponse } from "../api";

type OutputPanelProps = {
  running: boolean;
  response: RunResponse | null;
  error: string | null;
};

export function OutputPanel({ running, response, error }: OutputPanelProps) {
  const hasDiagnostics = Boolean(response && response.diagnostics.length > 0);
  const isSuccess = Boolean(!running && !error && response?.ok && !hasDiagnostics);

  return (
    <aside className="output-panel">
      <div className="output-header">
        <h2>Output</h2>
        <span className={`status-pill ${running ? "status-running" : isSuccess ? "status-ok" : hasDiagnostics || error ? "status-error" : "status-idle"}`}>
          {running ? "Running" : isSuccess ? "Success" : hasDiagnostics || error ? "Issues" : "Idle"}
        </span>
      </div>
      {running && <p className="muted">Running...</p>}
      {!running && error && <p className="error">IDE server error: {error}</p>}
      {!running && !error && !response && (
        <p className="muted">Output will appear here.</p>
      )}
      {!running && response && response.output.length > 0 && (
        <pre className="output-lines">
          {response.output.map((line, index) => (
            <span key={`${line}-${index}`}>{line}</span>
          ))}
        </pre>
      )}
      {!running && response?.ok && response.output.length === 0 && (
        <p className="muted">Program finished with no output.</p>
      )}
      {!running && response && response.diagnostics.length > 0 && (
        <section className="diagnostics">
          <h3>Diagnostics</h3>
          {response.diagnostics.map((diagnostic, index) => (
            <article className="diagnostic" key={`${diagnostic.message}-${index}`}>
              <div>{formatLocation(diagnostic.line, diagnostic.column)}</div>
              <p>{diagnostic.message}</p>
            </article>
          ))}
        </section>
      )}
    </aside>
  );
}

function formatLocation(line?: number | null, column?: number | null): string {
  if (line && column) {
    return `line ${line}, column ${column}`;
  }
  return "runtime";
}
