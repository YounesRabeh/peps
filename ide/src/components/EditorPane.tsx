import Editor, { type BeforeMount } from "@monaco-editor/react";
import { registerPepsLanguage } from "../pepsLanguage";

type EditorPaneProps = {
  source: string;
  onChange: (source: string) => void;
};

export function EditorPane({ source, onChange }: EditorPaneProps) {
  const handleBeforeMount: BeforeMount = (monaco) => {
    registerPepsLanguage(monaco);
  };

  return (
    <section className="editor-pane" aria-label="Peps editor">
      <Editor
        beforeMount={handleBeforeMount}
        language="peps"
        theme="peps-dark"
        value={source}
        onChange={(value) => onChange(value ?? "")}
        options={{
          automaticLayout: true,
          fontSize: 16,
          minimap: { enabled: false },
          tabSize: 4,
          wordWrap: "on"
        }}
      />
    </section>
  );
}
