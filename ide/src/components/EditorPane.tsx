import Editor, { type BeforeMount, type OnMount } from "@monaco-editor/react";
import { useEffect, useRef } from "react";
import type { editor as MonacoEditor } from "monaco-editor";
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
  renderWhitespace: "none" as const,
  mouseWheelZoom: true
};

const LETTER_FALLBACK_KEYS = new Set(["a", "A", "s", "S", "d", "D"]);

export function EditorPane({ source, onChange }: EditorPaneProps) {
  const editorRef = useRef<MonacoEditor.IStandaloneCodeEditor | null>(null);

  useEffect(() => {
    const onWindowKeyDown = (event: KeyboardEvent) => {
      const editor = editorRef.current;
      if (!editor || !editor.hasTextFocus()) {
        return;
      }
      if (event.ctrlKey || event.metaKey || event.altKey || event.isComposing) {
        return;
      }
      if (!LETTER_FALLBACK_KEYS.has(event.key)) {
        return;
      }

      event.preventDefault();
      event.stopImmediatePropagation();
      editor.trigger("keyboard", "type", { text: event.key });
    };

    window.addEventListener("keydown", onWindowKeyDown, true);
    return () => window.removeEventListener("keydown", onWindowKeyDown, true);
  }, []);

  const handleBeforeMount: BeforeMount = (monaco) => {
    registerPepsLanguage(monaco);
  };

  const handleMount: OnMount = (editor, monaco) => {
    editorRef.current = editor;
    monaco.editor.setTheme("peps-dark");

    editor.updateOptions(pepsEditorOptions);

    // Keep Ctrl+wheel zoom scoped to Monaco while pointer is over editor.
    const editorNode = editor.getDomNode();
    if (editorNode) {
      editorNode.addEventListener(
        "wheel",
        (event) => {
          if (event.ctrlKey) {
            event.preventDefault();
          }
        },
        { passive: false }
      );
    }

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
