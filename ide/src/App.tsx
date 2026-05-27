import { useState } from "react";
import { runPepsSource, type RunResponse } from "./api";
import { EditorPane } from "./components/EditorPane";
import { OutputPanel } from "./components/OutputPanel";
import { Toolbar } from "./components/Toolbar";
import { BASIC_SAMPLE } from "./examples";

export function App() {
  const [source, setSource] = useState(BASIC_SAMPLE);
  const [running, setRunning] = useState(false);
  const [response, setResponse] = useState<RunResponse | null>(null);
  const [error, setError] = useState<string | null>(null);

  async function handleRun() {
    setRunning(true);
    setError(null);
    setResponse(null);

    try {
      const result = await runPepsSource(source);
      setResponse(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setRunning(false);
    }
  }

  return (
    <main className="app-shell">
      <Toolbar running={running} onRun={handleRun} />
      <div className="workbench">
        <EditorPane source={source} onChange={setSource} />
        <OutputPanel running={running} response={response} error={error} />
      </div>
    </main>
  );
}
