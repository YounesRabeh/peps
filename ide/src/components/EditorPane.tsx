import Editor, { type BeforeMount, type OnMount } from "@monaco-editor/react";
import { registerPepsLanguage } from "../pepsLanguage";

type EditorPaneProps = {
  source: string;
  onChange: (source: string) => void;
};

const pepsEditorOptions = {
  automaticLayout: true,
  fontSize: 16,
  fontFamily: `"JetBrains Mono", "Fira Code", "Noto Color Emoji", "Segoe UI Emoji", "Apple Color Emoji", monospace`,
  minimap: { enabled: false },
  tabSize: 4,
  wordWrap: "on" as const,

  hover: {
    enabled: false
  },

  unicodeHighlight: {
    invisibleCharacters: false,
    ambiguousCharacters: false,
    nonBasicASCII: false,
    includeComments: false,
    includeStrings: false,
    allowedCharacters: {
      "\ufe0f": true, // emoji variation selector
      "\ufe0e": true, // text variation selector
      "\u200d": true, // zero-width joiner
      "\u20e3": true  // keycap combining mark
    }
  },

  renderControlCharacters: false,
  renderWhitespace: "none" as const
};

export function EditorPane({ source, onChange }: EditorPaneProps) {
  const handleBeforeMount: BeforeMount = (monaco) => {
    registerPepsLanguage(monaco);
  };

  const handleMount: OnMount = (editor, monaco) => {
    monaco.editor.setTheme("peps-dark");

    editor.updateOptions(pepsEditorOptions);

    console.log(
      "Peps Monaco unicodeHighlight option:",
      editor.getOption(monaco.editor.EditorOption.unicodeHighlight)
    );
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
        options={pepsEditorOptions}
      />
    </section>
  );
}