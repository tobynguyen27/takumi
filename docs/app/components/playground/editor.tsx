import { Editor } from "@monaco-editor/react";
import { shikiToMonaco } from "@shikijs/monaco";
import { useMemo, useRef } from "react";
import { createJavaScriptRegexEngine } from "shiki";
import { createHighlighterCore } from "shiki/core";
import reactTypings from "../../../node_modules/@types/react/index.d.ts?raw";
import cssTypings from "../../../node_modules/csstype/index.d.ts?raw";

const highlighter = await createHighlighterCore({
  themes: [import("shiki/themes/github-dark-default.mjs")],
  langs: [import("shiki/langs/tsx.mjs")],
  engine: createJavaScriptRegexEngine(),
  langAlias: {
    typescript: "tsx",
  },
});

export function ComponentEditor({
  code,
  setCode,
}: {
  code: string;
  setCode: (code: string) => void;
}) {
  const codeRef = useRef(code);

  const memorized = useMemo(
    () => (
      <Editor
        beforeMount={(monaco) => {
          monaco.languages.typescript.typescriptDefaults.setCompilerOptions({
            target: monaco.languages.typescript.ScriptTarget.Latest,
            allowNonTsExtensions: true,
            moduleResolution:
              monaco.languages.typescript.ModuleResolutionKind.NodeJs,
            module: monaco.languages.typescript.ModuleKind.CommonJS,
            noEmit: true,
            esModuleInterop: true,
            jsx: monaco.languages.typescript.JsxEmit.React,
            reactNamespace: "React",
            allowJs: true,
            typeRoots: ["node_modules/@types"],
          });

          monaco.languages.typescript.typescriptDefaults.addExtraLib(
            reactTypings,
            "file:///node_modules/react/index.d.ts",
          );

          monaco.languages.typescript.typescriptDefaults.addExtraLib(
            cssTypings,
            "file:///node_modules/csstype/index.d.ts",
          );

          shikiToMonaco(highlighter, monaco);
        }}
        width="100%"
        height="100%"
        language="typescript"
        theme="github-dark-default"
        options={{
          wordWrap: "on",
          tabSize: 2,
          minimap: {
            enabled: false,
          },
          stickyScroll: {
            enabled: false,
          },
          scrollbar: {
            useShadows: false,
          },
          fontSize: 16,
          scrollBeyondLastLine: false,
        }}
        loading="Launching editor..."
        defaultValue={codeRef.current}
        onChange={(value) => setCode(value ?? "")}
      />
    ),
    [setCode],
  );

  return memorized;
}
