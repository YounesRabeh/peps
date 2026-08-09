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
  const [outputHeight, setOutputHeight] = useState<number | null>(null);
  const [panelsVisible, setPanelsVisible] = useState(true);
  const [resizing, setResizing] = useState(false);
  const [resizingOutput, setResizingOutput] = useState(false);
  const workbenchRef = useRef<HTMLDivElement | null>(null);
  const sidebarRef = useRef<HTMLElement | null>(null);

  async function handleRun() {
    setPanelsVisible(true);
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

  function clampOutputHeight(requestedHeight: number) {
    const sidebar = sidebarRef.current;
    if (!sidebar) return Math.max(140, requestedHeight);

    const bounds = sidebar.getBoundingClientRect();
    const minimumOutputHeight = 140;
    const minimumDocsHeight = 220;
    const maximumOutputHeight = Math.max(
      minimumOutputHeight,
      bounds.height - minimumDocsHeight - 10,
    );
    return Math.min(maximumOutputHeight, Math.max(minimumOutputHeight, requestedHeight));
  }

  function setOutputHeightFromPointer(clientY: number) {
    const sidebar = sidebarRef.current;
    if (!sidebar) return;

    const bounds = sidebar.getBoundingClientRect();
    setOutputHeight(clampOutputHeight(clientY - bounds.top - 5));
  }

  function handlePanelDividerPointerDown(event: React.PointerEvent<HTMLButtonElement>) {
    event.currentTarget.setPointerCapture(event.pointerId);
    setResizingOutput(true);
    setOutputHeightFromPointer(event.clientY);
  }

  function handlePanelDividerPointerMove(event: React.PointerEvent<HTMLButtonElement>) {
    if (resizingOutput) setOutputHeightFromPointer(event.clientY);
  }

  function handlePanelDividerPointerUp(event: React.PointerEvent<HTMLButtonElement>) {
    if (event.currentTarget.hasPointerCapture(event.pointerId)) {
      event.currentTarget.releasePointerCapture(event.pointerId);
    }
    setResizingOutput(false);
  }

  function handlePanelDividerKeyDown(event: React.KeyboardEvent<HTMLButtonElement>) {
    if (event.key === "ArrowUp") {
      event.preventDefault();
      setOutputHeight((height) => clampOutputHeight((height ?? 260) - 24));
    } else if (event.key === "ArrowDown") {
      event.preventDefault();
      setOutputHeight((height) => clampOutputHeight((height ?? 260) + 24));
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
        className={
          resizing
            ? "workbench is-resizing-horizontal"
            : resizingOutput
              ? "workbench is-resizing-vertical"
              : "workbench"
        }
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
        {panelsVisible && <section
          className="runner-sidebar"
          id="runner-sidebar"
          aria-label="Run results and documentation"
          ref={sidebarRef}
          style={outputHeight === null ? undefined : { gridTemplateRows: `${outputHeight}px 10px minmax(220px, 1fr)` }}
        >
          <OutputPanel running={running} response={response} error={error} />
          <button
            aria-controls="output-panel docs-panel"
            aria-label="Resize output and documentation"
            aria-orientation="horizontal"
            aria-valuemin={140}
            aria-valuenow={Math.round(outputHeight ?? 260)}
            className="runner-divider"
            onKeyDown={handlePanelDividerKeyDown}
            onPointerDown={handlePanelDividerPointerDown}
            onPointerMove={handlePanelDividerPointerMove}
            onPointerUp={handlePanelDividerPointerUp}
            role="separator"
            type="button"
          />
          <DocsPanel onLoadExample={setSource} />
        </section>}
      </div>
    </main>
  );
}
