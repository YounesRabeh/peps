type ToolbarProps = {
  running: boolean;
  onRun: () => void;
  panelsVisible: boolean;
  onTogglePanels: () => void;
};

export function Toolbar({ running, onRun, panelsVisible, onTogglePanels }: ToolbarProps) {
  return (
    <header className="toolbar">
      <div className="toolbar-brand">
        <div className="toolbar-title">Peps IDE</div>
        <div className="toolbar-subtitle">Local Emoji Compiler</div>
      </div>
      <div className="toolbar-actions">
        <button className="panel-toggle-button" onClick={onTogglePanels} type="button">
          {panelsVisible ? "Hide panels" : "Show panels"}
        </button>
        <button className="run-button" disabled={running} onClick={onRun}>
          {running ? "Running..." : "Run ▶"}
        </button>
      </div>
    </header>
  );
}
