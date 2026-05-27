import Editor, { type BeforeMount, type OnMount } from "@monaco-editor/react";
import { registerPepsLanguage } from "../pepsLanguage";

type EditorPaneProps = {
  source: string;
  onChange: (source: string) => void;
};

export function EditorPane({ source, onChange }: EditorPaneProps) {
  const handleBeforeMount: BeforeMount = (monaco) => {
    registerPepsLanguage(monaco);
  };

  const handleMount: OnMount = (_editor, monaco) => {
    monaco.editor.setTheme("peps-dark");
  };

  return (
    <section className="editor-pane" aria-label="Peps editor">
      <Editor
        beforeMount={handleBeforeMount}
        onMount={handleMount}
        language="peps"
        theme="peps-dark"
        value={source}
        onChange={(value) => onChange(value ?? "")}
        options={{
          automaticLayout: true,
          fontSize: 16,
          fontFamily: `"Noto Color Emoji", "Segoe UI Emoji", "Apple Color Emoji", monospace`,
          minimap: { enabled: false },
          tabSize: 4,
          wordWrap: "on"
        }}
      />
    </section>
  );
}