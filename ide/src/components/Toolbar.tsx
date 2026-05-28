type ToolbarProps = {
  running: boolean;
  onRun: () => void;
};

export function Toolbar({ running, onRun }: ToolbarProps) {
  return (
    <header className="toolbar">
      <div className="toolbar-brand">
        <div className="toolbar-title">Peps IDE</div>
        <div className="toolbar-subtitle">Local Emoji Compiler</div>
      </div>
      <button className="run-button" disabled={running} onClick={onRun}>
        {running ? "Running..." : "Run ▶"}
      </button>
    </header>
  );
}
