import { useRef, useState } from "react";
import { runPepsSource, type RunResponse } from "./api";
import { EditorPane } from "./components/EditorPane";
import { DocsPanel } from "./components/DocsPanel";
import { OutputPanel } from "./components/OutputPanel";
import { Toolbar } from "./components/Toolbar";
import { BASIC_SAMPLE } from "./examples";

export function App() {
  const [source, setSource] = useState(BASIC_SAMPLE);
  const [running, setRunning] = useState(false);
  const [response, setResponse] = useState<RunResponse | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [sidebarWidth, setSidebarWidth] = useState(440);
  const [panelsVisible, setPanelsVisible] = useState(true);
  const [resizing, setResizing] = useState(false);
  const workbenchRef = useRef<HTMLDivElement | null>(null);

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

  function clampSidebarWidth(requestedWidth: number) {
    const workbench = workbenchRef.current;
    if (!workbench) {
      return Math.max(300, requestedWidth);
    }
    const bounds = workbench.getBoundingClientRect();
    const minimumEditorWidth = 280;
    const minimumSidebarWidth = 300;
    const maximumSidebarWidth = Math.max(minimumSidebarWidth, bounds.width - minimumEditorWidth - 10);
    return Math.min(maximumSidebarWidth, Math.max(minimumSidebarWidth, requestedWidth));
  }

  function setSidebarWidthFromPointer(clientX: number) {
    const workbench = workbenchRef.current;
    if (!workbench) {
      return;
    }
    const bounds = workbench.getBoundingClientRect();
    setSidebarWidth(clampSidebarWidth(bounds.right - clientX - 5));
  }

  function handleDividerPointerDown(event: React.PointerEvent<HTMLButtonElement>) {
    event.currentTarget.setPointerCapture(event.pointerId);
    setResizing(true);
    setSidebarWidthFromPointer(event.clientX);
  }

  function handleDividerPointerMove(event: React.PointerEvent<HTMLButtonElement>) {
    if (resizing) {
      setSidebarWidthFromPointer(event.clientX);
    }
  }

  function handleDividerPointerUp(event: React.PointerEvent<HTMLButtonElement>) {
    if (event.currentTarget.hasPointerCapture(event.pointerId)) {
      event.currentTarget.releasePointerCapture(event.pointerId);
    }
    setResizing(false);
  }

  function handleDividerKeyDown(event: React.KeyboardEvent<HTMLButtonElement>) {
    if (event.key === "ArrowLeft") {
      event.preventDefault();
      setSidebarWidth((width) => clampSidebarWidth(width + 24));
    } else if (event.key === "ArrowRight") {
      event.preventDefault();
      setSidebarWidth((width) => clampSidebarWidth(width - 24));
    }
  }

  return (
    <main className="app-shell">
      <Toolbar
        running={running}
        onRun={handleRun}
        panelsVisible={panelsVisible}
        onTogglePanels={() => setPanelsVisible((visible) => !visible)}
      />
      <div
        className={resizing ? "workbench is-resizing" : "workbench"}
        ref={workbenchRef}
        style={{
          gridTemplateColumns: panelsVisible
            ? `minmax(280px, 1fr) 10px ${sidebarWidth}px`
            : "minmax(0, 1fr)"
        }}
      >
        <EditorPane source={source} onChange={setSource} />
        {panelsVisible && (
          <button
            aria-controls="runner-sidebar"
            aria-label="Resize editor and panels"
            aria-orientation="vertical"
            aria-valuemin={300}
            aria-valuenow={Math.round(sidebarWidth)}
            className="workspace-divider"
            onKeyDown={handleDividerKeyDown}
            onPointerDown={handleDividerPointerDown}
            onPointerMove={handleDividerPointerMove}
            onPointerUp={handleDividerPointerUp}
            role="separator"
            type="button"
          />
        )}
        {panelsVisible && <section className="runner-sidebar" id="runner-sidebar" aria-label="Run results and documentation">
          <OutputPanel running={running} response={response} error={error} />
          <DocsPanel onLoadExample={setSource} />
        </section>}
      </div>
    </main>
  );
}
